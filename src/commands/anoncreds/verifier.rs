extern crate serde_json;

use errors::common::CommonError;
use errors::sovrin::SovrinError;

use services::anoncreds::AnoncredsService;
use services::pool::PoolService;
use services::wallet::WalletService;
use services::anoncreds::types::{
    ClaimDefinition,
    Schema,
    ProofRequestJson,
    ProofJson,
    RevocationRegistry};
use std::collections::HashMap;
use std::rc::Rc;
use utils::json::JsonDecodable;

pub enum VerifierCommand {
    VerifyProof(
        String, // proof request json
        String, // proof json
        String, // schemas json
        String, // claim defs jsons
        String, // revoc regs json
        Box<Fn(Result<bool, SovrinError>) + Send>)
}

pub struct VerifierCommandExecutor {
    anoncreds_service: Rc<AnoncredsService>,
    pool_service: Rc<PoolService>,
    wallet_service: Rc<WalletService>
}

impl VerifierCommandExecutor {
    pub fn new(anoncreds_service: Rc<AnoncredsService>,
               pool_service: Rc<PoolService>,
               wallet_service: Rc<WalletService>) -> VerifierCommandExecutor {
        VerifierCommandExecutor {
            anoncreds_service: anoncreds_service,
            pool_service: pool_service,
            wallet_service: wallet_service,
        }
    }

    pub fn execute(&self, command: VerifierCommand) {
        match command {
            VerifierCommand::VerifyProof(proof_request_json,
                                         proof_json, schemas_json,
                                         claim_defs_jsons, revoc_regs_json, cb) => {
                info!(target: "verifier_command_executor", "VerifyProof command received");
                self.verify_proof(&proof_request_json, &proof_json, &schemas_json,
                                  &claim_defs_jsons, &revoc_regs_json, cb);
            }
        };
    }

    fn verify_proof(&self,
                    proof_request_json: &str,
                    proof_json: &str,
                    schemas_json: &str,
                    claim_defs_jsons: &str,
                    revoc_regs_json: &str,
                    cb: Box<Fn(Result<bool, SovrinError>) + Send>) {
        let result = self._verify_proof(proof_request_json, proof_json, schemas_json, claim_defs_jsons, revoc_regs_json);
        cb(result)
    }

    fn _verify_proof(&self,
                     proof_request_json: &str,
                     proof_json: &str,
                     schemas_json: &str,
                     claim_defs_jsons: &str,
                     revoc_regs_json: &str) -> Result<bool, SovrinError> {
        let proof_req: ProofRequestJson = ProofRequestJson::from_json(proof_request_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Invalid proof_request_json: {}", err.to_string())))?;
        let schemas: HashMap<String, Schema> = serde_json::from_str(schemas_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Invalid schemas_json: {}", err.to_string())))?;
        let claim_defs: HashMap<String, ClaimDefinition> = serde_json::from_str(claim_defs_jsons)
            .map_err(|err| CommonError::InvalidStructure(format!("Invalid claim_defs_jsons: {}", err.to_string())))?;
        let revoc_regs: HashMap<String, RevocationRegistry> = serde_json::from_str(revoc_regs_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Invalid revoc_regs_json: {}", err.to_string())))?;
        let proof_claims: ProofJson = ProofJson::from_json(&proof_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Invalid proof_json: {}", err.to_string())))?;

        let result = self.anoncreds_service.verifier.verify(&proof_claims,
                                                            &proof_req.nonce,
                                                            &claim_defs,
                                                            &revoc_regs,
                                                            &schemas)?;

        Ok(result)
    }
}