#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Address, Vec, Symbol, symbol_short};

// Structure to store access log entries
#[contracttype]
#[derive(Clone)]
pub struct AccessLog {
    pub accessor: Address,      // Who accessed the record
    pub timestamp: u64,          // When it was accessed
    pub access_count: u64,       // Running count of accesses
}

// Enum for mapping patient to their authorized providers
#[contracttype]
pub enum AuthorizedProviders {
    Patient(Address)
}

// Symbol for access log count
const ACCESS_COUNT: Symbol = symbol_short!("ACC_CNT");

#[contract]
pub struct MedicalRecordsContract;

#[contractimpl]
impl MedicalRecordsContract {
    
    /// Function 1: Grant access to a healthcare provider
    /// Patient grants permission to a specific provider to access their medical records
    pub fn grant_access(env: Env, patient: Address, provider: Address) {
        // Verify that the caller is the patient
        patient.require_auth();
        
        // Get existing authorized providers or create new vector
        let key = AuthorizedProviders::Patient(patient.clone());
        let mut providers: Vec<Address> = env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));
        
        // Check if provider is already authorized
        if !providers.contains(&provider) {
            providers.push_back(provider.clone());
            env.storage().persistent().set(&key, &providers);
            env.storage().persistent().extend_ttl(&key, 5000, 5000);
            
            log!(&env, "Access granted to provider: {:?}", provider);
        } else {
            log!(&env, "Provider already has access");
        }
    }
    
    /// Function 2: Revoke access from a healthcare provider
    /// Patient revokes permission from a previously authorized provider
    pub fn revoke_access(env: Env, patient: Address, provider: Address) {
        // Verify that the caller is the patient
        patient.require_auth();
        
        let key = AuthorizedProviders::Patient(patient.clone());
        let mut providers: Vec<Address> = env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));
        
        // Find and remove the provider
        let mut new_providers = Vec::new(&env);
        for i in 0..providers.len() {
            let p = providers.get(i).unwrap();
            if p != provider {
                new_providers.push_back(p);
            }
        }
        
        env.storage().persistent().set(&key, &new_providers);
        env.storage().persistent().extend_ttl(&key, 5000, 5000);
        
        log!(&env, "Access revoked from provider: {:?}", provider);
    }
    
    /// Function 3: Access medical records (creates audit trail)
    /// Healthcare provider accesses patient records - this logs the access
    pub fn access_records(env: Env, patient: Address, provider: Address) {
        // Verify that the caller is the provider
        provider.require_auth();
        
        // Check if provider is authorized
        let key = AuthorizedProviders::Patient(patient.clone());
        let providers: Vec<Address> = env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));
        
        if !providers.contains(&provider) {
            log!(&env, "Unauthorized access attempt by: {:?}", provider);
            panic!("Provider is not authorized to access these records");
        }
        
        // Create audit log entry
        let timestamp = env.ledger().timestamp();
        let count_key = (Symbol::new(&env, "COUNT"), patient.clone());
        let mut count: u64 = env.storage().instance().get(&count_key).unwrap_or(0);
        count += 1;
        
        let log_entry = AccessLog {
            accessor: provider.clone(),
            timestamp,
            access_count: count,
        };
        
        // Store the access log for this specific patient
        let log_key = (Symbol::new(&env, "LOG"), patient.clone());
        env.storage().instance().set(&log_key, &log_entry);
        env.storage().instance().set(&count_key, &count);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Records accessed by: {:?} at timestamp: {}", provider, timestamp);
    }
    
    /// Function 4: View audit trail
    /// Returns the most recent access log entry (returns Option to handle no logs case)
    pub fn view_audit_trail(env: Env, patient: Address) -> Option<AccessLog> {
        let log_key = (Symbol::new(&env, "LOG"), patient);
        
        env.storage().instance().get(&log_key)
    }
}