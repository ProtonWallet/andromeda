use bdk::bitcoin::ScriptBuf;

#[derive(Clone, Debug)]
pub struct TxBuilder {
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

    // Add a recipient to the internal list.
    // pub(crate) fn add_recipient(&self, script: Arc<Script>, amount: u64) -> Arc<Self> {
    //     let mut recipients: Vec<(ScriptBuf, u64)> = self.recipients.clone();
    //     recipients.append(&mut vec![(script.0.clone(), amount)]);

    //     Arc::new(TxBuilder {
    //         recipients,
    //         ..self.clone()
    //     })
    // }
}
