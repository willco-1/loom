use std::collections::BTreeMap;

use alloy_primitives::{Address, Bytes, B256, U256};
use alloy_rpc_types_trace::geth::AccountState;
use alloy_sol_types::SolCall;
use lazy_static::lazy_static;

use defi_abi::uniswap3::IUniswapV3Pool;
use defi_types::GethStateUpdate;

use crate::protocols::helper::get_uniswap3pool_address;
use crate::protocols::match_abi;
use crate::protocols::protocol::Protocol;

lazy_static! {
    static ref CUSTOM_QUOTER_ADDRESS : Address = "0x0000000000000000000000000000000000003333".parse().unwrap();
    static ref CUSTOM_QUOTER_CODE: Bytes = "0x608060405234801561001057600080fd5b50600436106100675760003560e01c8063a0dfc0de11610050578063a0dfc0de146100ad578063c45a0155146100c0578063fa461e33146100c857610067565b80634aa4a4fc1461006c5780639084b37b1461008a575b600080fd5b6100746100dd565b60405161008191906114d2565b60405180910390f35b61009d61009836600461132c565b610101565b604051610081949392919061158f565b61009d6100bb36600461132c565b61033d565b610074610502565b6100db6100d636600461122b565b610526565b005b7f000000000000000000000000000000000000000000000000000000000000000081565b6000806000806000856040015173ffffffffffffffffffffffffffffffffffffffff16866020015173ffffffffffffffffffffffffffffffffffffffff161090506000866000015190508660a0015173ffffffffffffffffffffffffffffffffffffffff16600014156101775760608701516000555b60005a90508173ffffffffffffffffffffffffffffffffffffffff1663128acb0830856101a78c606001516106ab565b6000038c60a0015173ffffffffffffffffffffffffffffffffffffffff166000146101d6578c60a001516101fc565b876101f55773fffd8963efd1fc6a506488495d951d5263988d256101fc565b6401000276a45b8d604001518e608001518f6020015160405160200161021d9392919061146c565b6040516020818303038152906040526040518663ffffffff1660e01b815260040161024c9594939291906114f3565b6040805180830381600087803b15801561026557600080fd5b505af19250505080156102b3575060408051601f3d9081017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe01682019092526102b091810190611208565b60015b610330573d8080156102e1576040519150601f19603f3d011682016040523d82523d6000602084013e6102e6565b606091505b505a820394508860a0015173ffffffffffffffffffffffffffffffffffffffff166000141561031457600080555b61031f8184876106dd565b975097509750975050505050610336565b50505050505b9193509193565b6000806000806000856040015173ffffffffffffffffffffffffffffffffffffffff16866020015173ffffffffffffffffffffffffffffffffffffffff1610905060008660000151905060005a90508173ffffffffffffffffffffffffffffffffffffffff1663128acb0830856103b78c606001516106ab565b60a08d015173ffffffffffffffffffffffffffffffffffffffff16156103e1578c60a00151610407565b876104005773fffd8963efd1fc6a506488495d951d5263988d25610407565b6401000276a45b8d602001518e608001518f604001516040516020016104289392919061146c565b6040516020818303038152906040526040518663ffffffff1660e01b81526004016104579594939291906114f3565b6040805180830381600087803b15801561047057600080fd5b505af19250505080156104be575060408051601f3d9081017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe01682019092526104bb91810190611208565b60015b610330573d8080156104ec576040519150601f19603f3d011682016040523d82523d6000602084013e6104f1565b606091505b505a8203945061031f8184876106dd565b7f000000000000000000000000000000000000000000000000000000000000000081565b60008313806105355750600082135b61053e57600080fd5b600080600061054c846107b1565b9250925092506000806000808913610597578573ffffffffffffffffffffffffffffffffffffffff168573ffffffffffffffffffffffffffffffffffffffff1610888a6000036105cc565b8473ffffffffffffffffffffffffffffffffffffffff168673ffffffffffffffffffffffffffffffffffffffff161089896000035b92509250925060003390506000808273ffffffffffffffffffffffffffffffffffffffff16633850c7bd6040518163ffffffff1660e01b815260040160e06040518083038186803b15801561062057600080fd5b505afa158015610634573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610658919061134e565b505050505091509150851561067e57604051848152826020820152816040820152606081fd5b6000541561069457600054841461069457600080fd5b604051858152826020820152816040820152606081fd5b60007f800000000000000000000000000000000000000000000000000000000000000082106106d957600080fd5b5090565b6000806000806000808773ffffffffffffffffffffffffffffffffffffffff16633850c7bd6040518163ffffffff1660e01b815260040160e06040518083038186803b15801561072c57600080fd5b505afa158015610740573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610764919061134e565b5093965061077994508d93506107e292505050565b919750955090506107a173ffffffffffffffffffffffffffffffffffffffff891683836108a3565b9350869250505093509350935093565b600080806107bf8482610efb565b92506107cc846014610ffb565b90506107d9846017610efb565b91509193909250565b6000806000835160601461088257604484511015610835576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161082c90611558565b60405180910390fd5b6004840193508380602001905181019061084f91906112b9565b6040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161082c9190611545565b8380602001905181019061089691906113e5565b9250925092509193909250565b60008060008060008060008060088b73ffffffffffffffffffffffffffffffffffffffff1663d0c93a7c6040518163ffffffff1660e01b815260040160206040518083038186803b1580156108f757600080fd5b505afa15801561090b573d6000803e3d6000fd5b505050506040513d602081101561092157600080fd5b5051600290810b908c900b8161093357fe5b0560020b901d905060006101008c73ffffffffffffffffffffffffffffffffffffffff1663d0c93a7c6040518163ffffffff1660e01b815260040160206040518083038186803b15801561098657600080fd5b505afa15801561099a573d6000803e3d6000fd5b505050506040513d60208110156109b057600080fd5b5051600290810b908d900b816109c257fe5b0560020b816109cd57fe5b079050600060088d73ffffffffffffffffffffffffffffffffffffffff1663d0c93a7c6040518163ffffffff1660e01b815260040160206040518083038186803b158015610a1a57600080fd5b505afa158015610a2e573d6000803e3d6000fd5b505050506040513d6020811015610a4457600080fd5b5051600290810b908d900b81610a5657fe5b0560020b901d905060006101008e73ffffffffffffffffffffffffffffffffffffffff1663d0c93a7c6040518163ffffffff1660e01b815260040160206040518083038186803b158015610aa957600080fd5b505afa158015610abd573d6000803e3d6000fd5b505050506040513d6020811015610ad357600080fd5b5051600290810b908e900b81610ae557fe5b0560020b81610af057fe5b07905060008160ff166001901b8f73ffffffffffffffffffffffffffffffffffffffff16635339c296856040518263ffffffff1660e01b8152600401808260010b815260200191505060206040518083038186803b158015610b5157600080fd5b505afa158015610b65573d6000803e3d6000fd5b505050506040513d6020811015610b7b57600080fd5b505116118015610c0e57508d73ffffffffffffffffffffffffffffffffffffffff1663d0c93a7c6040518163ffffffff1660e01b815260040160206040518083038186803b158015610bcc57600080fd5b505afa158015610be0573d6000803e3d6000fd5b505050506040513d6020811015610bf657600080fd5b5051600290810b908d900b81610c0857fe5b0760020b155b8015610c1f57508b60020b8d60020b135b945060008360ff166001901b8f73ffffffffffffffffffffffffffffffffffffffff16635339c296876040518263ffffffff1660e01b8152600401808260010b815260200191505060206040518083038186803b158015610c7f57600080fd5b505afa158015610c93573d6000803e3d6000fd5b505050506040513d6020811015610ca957600080fd5b505116118015610d3c57508d73ffffffffffffffffffffffffffffffffffffffff1663d0c93a7c6040518163ffffffff1660e01b815260040160206040518083038186803b158015610cfa57600080fd5b505afa158015610d0e573d6000803e3d6000fd5b505050506040513d6020811015610d2457600080fd5b5051600290810b908e900b81610d3657fe5b0760020b155b8015610d4d57508b60020b8d60020b125b95508160010b8460010b1280610d7957508160010b8460010b148015610d7957508060ff168360ff1611155b15610d8f57839950829750819850809650610d9c565b8199508097508398508296505b50507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff60ff87161b9150505b8560010b8760010b13610ed3578560010b8760010b1415610e0d577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff60ff858103161c165b6000818c73ffffffffffffffffffffffffffffffffffffffff16635339c2968a6040518263ffffffff1660e01b8152600401808260010b815260200191505060206040518083038186803b158015610e6457600080fd5b505afa158015610e78573d6000803e3d6000fd5b505050506040513d6020811015610e8e57600080fd5b5051169050610e9c816110eb565b61ffff16989098019750506001909501947fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff610dc8565b8115610ee0576001880397505b8215610eed576001880397505b505050505050509392505050565b600081826014011015610f6f57604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601260248201527f746f416464726573735f6f766572666c6f770000000000000000000000000000604482015290519081900360640190fd5b8160140183511015610fe257604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601560248201527f746f416464726573735f6f75744f66426f756e64730000000000000000000000604482015290519081900360640190fd5b5001602001516c01000000000000000000000000900490565b60008182600301101561106f57604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601160248201527f746f55696e7432345f6f766572666c6f77000000000000000000000000000000604482015290519081900360640190fd5b81600301835110156110e257604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601460248201527f746f55696e7432345f6f75744f66426f756e6473000000000000000000000000604482015290519081900360640190fd5b50016003015190565b6000805b8215611124577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8301909216916001016110ef565b90505b919050565b80356111278161165a565b8051600281900b811461112757600080fd5b600060c0828403121561115a578081fd5b60405160c0810181811067ffffffffffffffff8211171561117757fe5b60405290508082356111888161165a565b815260208301356111988161165a565b602082015260408301356111ab8161165a565b6040820152606083810135908201526111c6608084016111f5565b60808201526111d760a0840161112c565b60a08201525092915050565b805161ffff8116811461112757600080fd5b803562ffffff8116811461112757600080fd5b6000806040838503121561121a578182fd5b505080516020909101519092909150565b60008060006060848603121561123f578081fd5b8335925060208401359150604084013567ffffffffffffffff811115611263578182fd5b8401601f81018613611273578182fd5b8035611286611281826115ea565b6115c6565b81815287602083850101111561129a578384fd5b8160208401602083013783602083830101528093505050509250925092565b6000602082840312156112ca578081fd5b815167ffffffffffffffff8111156112e0578182fd5b8201601f810184136112f0578182fd5b80516112fe611281826115ea565b818152856020838501011115611312578384fd5b61132382602083016020860161162a565b95945050505050565b600060c0828403121561133d578081fd5b6113478383611149565b9392505050565b600080600080600080600060e0888a031215611368578283fd5b87516113738161165a565b965061138160208901611137565b955061138f604089016111e3565b945061139d606089016111e3565b93506113ab608089016111e3565b925060a088015160ff811681146113c0578283fd5b60c089015190925080151581146113d5578182fd5b8091505092959891949750929550565b6000806000606084860312156113f9578081fd5b83519250602084015161140b8161165a565b915061141960408501611137565b90509250925092565b6000815180845261143a81602086016020860161162a565b601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0169290920160200192915050565b606093841b7fffffffffffffffffffffffffffffffffffffffff000000000000000000000000908116825260e89390931b7fffffff0000000000000000000000000000000000000000000000000000000000166014820152921b166017820152602b0190565b73ffffffffffffffffffffffffffffffffffffffff91909116815260200190565b600073ffffffffffffffffffffffffffffffffffffffff8088168352861515602084015285604084015280851660608401525060a0608083015261153a60a0830184611422565b979650505050505050565b6000602082526113476020830184611422565b60208082526010908201527f556e6578706563746564206572726f7200000000000000000000000000000000604082015260600190565b93845273ffffffffffffffffffffffffffffffffffffffff92909216602084015263ffffffff166040830152606082015260800190565b60405181810167ffffffffffffffff811182821017156115e257fe5b604052919050565b600067ffffffffffffffff8211156115fe57fe5b50601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe01660200190565b60005b8381101561164557818101518382015260200161162d565b83811115611654576000848401525b50505050565b73ffffffffffffffffffffffffffffffffffffffff8116811461167c57600080fd5b5056fea164736f6c6343000706000a".parse().unwrap();
}

pub struct UniswapV3Protocol {}

impl UniswapV3Protocol {
    pub fn get_pool_address_for_tokens(token0: Address, token1: Address, fee: u32) -> Address {
        let uni3_factory_address: Address = "0x1F98431c8aD98523631AE4a59f267346ea31F984".parse().unwrap();
        let init_code: B256 = "e34f199b19b2b4f47f68442619d555527d244f78a3297ea89325f843f87b8b54".parse().unwrap();

        get_uniswap3pool_address(token0, token1, fee, uni3_factory_address, init_code)
    }

    pub fn get_custom_quoter_code() -> Bytes {
        CUSTOM_QUOTER_CODE.clone()
    }

    pub fn get_custom_quoter_address() -> Address {
        *CUSTOM_QUOTER_ADDRESS
    }

    pub fn get_quoter_v3_state() -> GethStateUpdate {
        let mut state: GethStateUpdate = BTreeMap::new();

        let acc_state =
            AccountState { balance: Some(U256::ZERO), code: Some(CUSTOM_QUOTER_CODE.clone()), nonce: None, storage: BTreeMap::new() };

        state.insert(Self::get_custom_quoter_address(), acc_state);

        state
    }

    pub fn is_code(code: &Bytes) -> bool {
        match_abi(code, vec![IUniswapV3Pool::swapCall::SELECTOR, IUniswapV3Pool::mintCall::SELECTOR, IUniswapV3Pool::collectCall::SELECTOR])
    }
}

impl Protocol for UniswapV3Protocol {
    fn get_pool_address_vec_for_tokens(token0: Address, token1: Address) -> Vec<Address> {
        let uni3_factory_address: Address = "0x1F98431c8aD98523631AE4a59f267346ea31F984".parse().unwrap();
        let init_code: B256 = "e34f199b19b2b4f47f68442619d555527d244f78a3297ea89325f843f87b8b54".parse().unwrap();

        let pair_address0 = get_uniswap3pool_address(token0, token1, 100, uni3_factory_address, init_code);
        let pair_address1 = get_uniswap3pool_address(token0, token1, 500, uni3_factory_address, init_code);
        let pair_address2 = get_uniswap3pool_address(token0, token1, 3000, uni3_factory_address, init_code);
        let pair_address3 = get_uniswap3pool_address(token0, token1, 10000, uni3_factory_address, init_code);

        vec![pair_address0, pair_address1, pair_address2, pair_address3]
    }
}
