use std::sync::Arc;

use bdk::bitcoin::ScriptBuf;

/// A Bitcoin script.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Script(pub(crate) ScriptBuf);
impl Script {
    pub fn new(raw_output_script: Vec<u8>) -> Self {
        let script: ScriptBuf = raw_output_script.into();
        Script(script)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }
}

//
/// A transaction builder.
/// After creating the TxBuilder, you set options on it until finally calling finish to consume the builder and generate the transaction.
/// Each method on the TxBuilder returns an instance of a new TxBuilder with the option set/added.
#[derive(Clone, Debug)]
pub(crate) struct TxBuilder {
    pub(crate) recipients: Vec<(ScriptBuf, u64)>,
    // pub(crate) utxos: Vec<OutPoint>,
    // pub(crate) unspendable: HashSet<OutPoint>,
    // pub(crate) change_policy: ChangeSpendPolicy,
    // pub(crate) manually_selected_only: bool,
    pub(crate) fee_rate: Option<f32>,
    // pub(crate) fee_absolute: Option<u64>,
    // pub(crate) drain_wallet: bool,
    // pub(crate) drain_to: Option<BdkScript>,
    // pub(crate) rbf: Option<RbfValue>,
    // pub(crate) data: Vec<u8>,
}

impl TxBuilder {
    pub(crate) fn new() -> Self {
        TxBuilder {
            recipients: Vec::new(),
            // utxos: Vec::new(),
            // unspendable: HashSet::new(),
            // change_policy: ChangeSpendPolicy::ChangeAllowed,
            // manually_selected_only: false,
            fee_rate: None,
            // fee_absolute: None,
            // drain_wallet: false,
            // drain_to: None,
            // rbf: None,
            // data: Vec::new(),
        }
    }

    /// Add a recipient to the internal list.
    pub(crate) fn add_recipient(&self, script: Arc<Script>, amount: u64) -> Arc<Self> {
        let mut recipients: Vec<(ScriptBuf, u64)> = self.recipients.clone();
        recipients.append(&mut vec![(script.0.clone(), amount)]);

        Arc::new(TxBuilder {
            recipients,
            ..self.clone()
        })
    }
}

fn sign_transaction() {

    // TxBuilder {
    //     recipients: Vec::new(),
    //     // utxos: Vec::new(),
    //     // unspendable: HashSet::new(),
    //     // change_policy: ChangeSpendPolicy::ChangeAllowed,
    //     // manually_selected_only: false,
    //     fee_rate: None,
    //     // fee_absolute: None,
    //     // drain_wallet: false,
    //     // drain_to: None,
    //     // rbf: None,
    //     // data: Vec::new(),
    // }
}
