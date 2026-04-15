use primitives::types::{Account, Address, U256};

pub fn genesis_accounts_info() -> Vec<(Address, Account)> {
    vec![
        (
            Address::from_hex("28dcb1338b900419cd613a8fb273ae36e7ec2b1d".to_string()).unwrap(),
            Account {
                nonce: 0,
                balance: U256::from(10000000),
            },
        ),
        (
            Address::from_hex("0534501c34f5a0f3fa43dc5d78e619be7edfa21a".to_string()).unwrap(),
            Account {
                nonce: 0,
                balance: U256::from(12000000),
            },
        ),
    ]
}
