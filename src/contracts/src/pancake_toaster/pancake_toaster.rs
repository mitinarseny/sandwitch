pub use pancake_toaster::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod pancake_toaster {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    use ethers::contract::{
        builders::{ContractCall, Event},
        Contract, Lazy,
    };
    use ethers::core::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers::providers::Middleware;
    #[doc = "PancakeToaster was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"contract IPancakeFactory\",\"name\":\"_factory\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"type\":\"error\",\"name\":\"InsufficientBalance\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InsufficientInputAmount\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InsufficientLiquidity\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InsufficientOutputAmount\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"contract IERC20\",\"name\":\"token\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"type\":\"error\",\"name\":\"InsufficientTokenBalance\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidPath\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"SlippageExhausted\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"Uncled\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"contract IERC20\",\"name\":\"tokenIn\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"contract IERC20\",\"name\":\"tokenOut\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"backRunSwapAll\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"amountOut\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"factory\",\"outputs\":[{\"internalType\":\"contract IPancakeFactory\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountOut\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"ETHIn\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"contract IERC20[]\",\"name\":\"path\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"indexIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"parentBlockHash\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"frontRunSwap\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"ourAmountIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ourAmountOut\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amountOut\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"ETHIn\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"contract IERC20[]\",\"name\":\"path\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"indexIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"parentBlockHash\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"frontRunSwapExt\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"ourAmountIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ourAmountOut\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"newReserveIn\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"newReserveOut\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"owner\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"renounceOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferOwnership\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static PANCAKETOASTER_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static PANCAKETOASTER_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x60a06040523480156200001157600080fd5b5060405162002997380380620029978339818101604052810190620000379190620001dc565b620000576200004b6200009260201b60201c565b6200009a60201b60201c565b8073ffffffffffffffffffffffffffffffffffffffff1660808173ffffffffffffffffffffffffffffffffffffffff1681525050506200020e565b600033905090565b60008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff169050816000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e060405160405180910390a35050565b600080fd5b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000620001908262000163565b9050919050565b6000620001a48262000183565b9050919050565b620001b68162000197565b8114620001c257600080fd5b50565b600081519050620001d681620001ab565b92915050565b600060208284031215620001f557620001f46200015e565b5b60006200020584828501620001c5565b91505092915050565b60805161274a6200024d60003960008181610381015281816104430152818161061901528181610782015281816108230152610b85015261274a6000f3fe608060405234801561001057600080fd5b506004361061007d5760003560e01c8063aadaa0971161005b578063aadaa097146100db578063c45a01551461010b578063c66e13cc14610129578063f2fde38b1461015c5761007d565b806331d0baf414610082578063715018a6146100b35780638da5cb5b146100bd575b600080fd5b61009c60048036038101906100979190611aa7565b610178565b6040516100aa929190611b78565b60405180910390f35b6100bb610502565b005b6100c5610516565b6040516100d29190611bb0565b60405180910390f35b6100f560048036038101906100f09190611c09565b61053f565b6040516101029190611c49565b60405180910390f35b610113610780565b6040516101209190611cc3565b60405180910390f35b610143600480360381019061013e9190611aa7565b6107a4565b6040516101539493929190611cde565b60405180910390f35b61017660048036038101906101719190611d23565b610881565b005b600080828060014361018a9190611d7f565b40146101c2576040517f7797ae6d00000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b868690506001866101d39190611db3565b1061020a576040517f20db826700000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b89886102b7578787600081811061022457610223611de7565b5b90506020020160208101906102399190611e16565b73ffffffffffffffffffffffffffffffffffffffff166370a082318d6040518263ffffffff1660e01b81526004016102719190611bb0565b602060405180830381865afa15801561028e573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906102b29190611e58565b6102d0565b8b73ffffffffffffffffffffffffffffffffffffffff16315b1015610313578a6040517f897f6c5800000000000000000000000000000000000000000000000000000000815260040161030a9190611bb0565b60405180910390fd5b60008511156103c8576103c58a888860009060018a6103329190611db3565b9261033f93929190611e8f565b80806020026020016040519081016040528093929190818152602001838360200280828437600081840152601f19601f820116905080830192505050505050507f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff166109049092919063ffffffff16565b99505b868690506002866103d99190611db3565b101561048a576104878988886001896103f29190611db3565b90809261040193929190611e8f565b80806020026020016040519081016040528093929190818152602001838360200280828437600081840152601f19601f820116905080830192505050505050507f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff166109d99092919063ffffffff16565b98505b6104f08a8a8989898181106104a2576104a1611de7565b5b90506020020160208101906104b79190611e16565b8a8a60018b6104c69190611db3565b8181106104d6576104d5611de7565b5b90506020020160208101906104eb9190611e16565b610ab2565b92509250509850989650505050505050565b61050a610d4b565b6105146000610dc9565b565b60008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905090565b6000610549610d4b565b60008373ffffffffffffffffffffffffffffffffffffffff166370a08231306040518263ffffffff1660e01b81526004016105849190611bb0565b602060405180830381865afa1580156105a1573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906105c59190611e58565b90506000810361060c57306040517f897f6c580000000000000000000000000000000000000000000000000000000081526004016106039190611bb0565b60405180910390fd5b60008060008061065d88887f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff16610e8d9092919063ffffffff16565b9350935093509350610670858585610f5e565b955061069d82868a73ffffffffffffffffffffffffffffffffffffffff166110279092919063ffffffff16565b600080826106ad576000886106b1565b8760005b915091508373ffffffffffffffffffffffffffffffffffffffff1663022c0d9f838333600067ffffffffffffffff8111156106ef576106ee611eca565b5b6040519080825280601f01601f1916602001820160405280156107215781602001600182028036833780820191505090505b506040518563ffffffff1660e01b81526004016107419493929190611f89565b600060405180830381600087803b15801561075b57600080fd5b505af115801561076f573d6000803e3d6000fd5b505050505050505050505092915050565b7f000000000000000000000000000000000000000000000000000000000000000081565b6000806000806107ba8c8c8c8c8c8c8c8c610178565b80945081955050506108678888888181106107d8576107d7611de7565b5b90506020020160208101906107ed9190611e16565b898960018a6107fc9190611db3565b81811061080c5761080b611de7565b5b90506020020160208101906108219190611e16565b7f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff16610e8d9092919063ffffffff16565b905050809250819350505098509850985098945050505050565b610889610d4b565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16036108f8576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016108ef90612058565b60405180910390fd5b61090181610dc9565b50565b6000600282511015610942576040517f20db826700000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b60005b600183516109539190611d7f565b8110156109ce576000806109a88786858151811061097457610973611de7565b5b60200260200101518760018761098a9190611db3565b8151811061099b5761099a611de7565b5b6020026020010151610e8d565b5050915091506109b9868383610f5e565b95505050806109c790612078565b9050610945565b508290509392505050565b6000600282511015610a17576040517f20db826700000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b600060018351610a279190611d7f565b90505b6000811115610aa757600080610a818786600186610a489190611d7f565b81518110610a5957610a58611de7565b5b6020026020010151878681518110610a7457610a73611de7565b5b6020026020010151610e8d565b505091509150610a928683836110ad565b9550505080610aa0906120c0565b9050610a2a565b508290509392505050565b60008060008473ffffffffffffffffffffffffffffffffffffffff166370a08231336040518263ffffffff1660e01b8152600401610af09190611bb0565b602060405180830381865afa158015610b0d573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610b319190611e58565b905060008103610b7857336040517f897f6c58000000000000000000000000000000000000000000000000000000008152600401610b6f9190611bb0565b60405180910390fd5b600080600080610bc989897f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff16610e8d9092919063ffffffff16565b9350935093509350610bef85610be18d8d888861117f565b61124890919063ffffffff16565b9650610bfc878585610f5e565b955060008603610c38576040517fca46902900000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b610c653383898c73ffffffffffffffffffffffffffffffffffffffff16611261909392919063ffffffff16565b60008082610c7557600088610c79565b8760005b915091508373ffffffffffffffffffffffffffffffffffffffff1663022c0d9f838330600067ffffffffffffffff811115610cb757610cb6611eca565b5b6040519080825280601f01601f191660200182016040528015610ce95781602001600182028036833780820191505090505b506040518563ffffffff1660e01b8152600401610d099493929190611f89565b600060405180830381600087803b158015610d2357600080fd5b505af1158015610d37573d6000803e3d6000fd5b505050505050505050505094509492505050565b610d536112ea565b73ffffffffffffffffffffffffffffffffffffffff16610d71610516565b73ffffffffffffffffffffffffffffffffffffffff1614610dc7576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610dbe90612135565b60405180910390fd5b565b60008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff169050816000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e060405160405180910390a35050565b600080600080610e9e8787876112f2565b80925081935050508173ffffffffffffffffffffffffffffffffffffffff16630902f1ac6040518163ffffffff1660e01b8152600401606060405180830381865afa158015610ef1573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610f1591906121d7565b826dffffffffffffffffffffffffffff169250816dffffffffffffffffffffffffffff1691505080945081955050508015610f5557828480945081955050505b93509350935093565b6000808403610f99576040517f098fb56100000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b6000831480610fa85750600082145b15610fdf576040517fbb55fd2700000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b6126f784610fed919061222a565b93508361271084610ffe919061222a565b6110089190611db3565b8483611014919061222a565b61101e919061229b565b90509392505050565b6110a88363a9059cbb60e01b84846040516024016110469291906122cc565b604051602081830303815290604052907bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19166020820180517bffffffffffffffffffffffffffffffffffffffffffffffffffffffff838183161783525050505061134f565b505050565b60008084036110e8576040517f42301c2300000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b60008314806110f75750600082145b1561112e576040517fbb55fd2700000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b60016126f7858461113f9190611d7f565b611149919061222a565b6127108686611158919061222a565b611162919061222a565b61116c919061229b565b6111769190611db3565b90509392505050565b60006126f76002611190919061222a565b856126f761119e919061222a565b8461271060026111ae919061222a565b6111b8919061222a565b611220886126f76111c9919061222a565b88878961271060046111db919061222a565b6111e5919061222a565b6111ef919061222a565b6111f9919061229b565b6112039190611db3565b896126f7611211919061222a565b61121b919061222a565b611416565b61122a9190611d7f565b6112349190611d7f565b61123e919061229b565b9050949350505050565b60008183106112575781611259565b825b905092915050565b6112e4846323b872dd60e01b858585604051602401611282939291906122f5565b604051602081830303815290604052907bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19166020820180517bffffffffffffffffffffffffffffffffffffffffffffffffffffffff838183161783525050505061134f565b50505050565b600033905090565b600080600080611302868661150f565b915091506113118783836115cf565b8273ffffffffffffffffffffffffffffffffffffffff168773ffffffffffffffffffffffffffffffffffffffff161415935093505050935093915050565b60006113b1826040518060400160405280602081526020017f5361666545524332303a206c6f772d6c6576656c2063616c6c206661696c65648152508573ffffffffffffffffffffffffffffffffffffffff166116889092919063ffffffff16565b905060008151111561141157808060200190518101906113d19190612341565b611410576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401611407906123e0565b60405180910390fd5b5b505050565b6000808203611428576000905061150a565b60006001611435846116a0565b901c6001901b9050600181848161144f5761144e61226c565b5b048201901c905060018184816114685761146761226c565b5b048201901c905060018184816114815761148061226c565b5b048201901c9050600181848161149a5761149961226c565b5b048201901c905060018184816114b3576114b261226c565b5b048201901c905060018184816114cc576114cb61226c565b5b048201901c905060018184816114e5576114e461226c565b5b048201901c905061150681828581611500576114ff61226c565b5b04611248565b9150505b919050565b6000808273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff160361154a57600080fd5b8273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff1610611584578284611587565b83835b8092508193505050600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16036115c857600080fd5b9250929050565b60008173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff161061160957600080fd5b83838360405160200161161d92919061245a565b604051602081830303815290604052805190602001207fa5934690703a592a07e841ca29d5e5c79b5e22ed4749057bb216dc31100be1c060405160200161166693929190612515565b6040516020818303038152906040528051906020012060001c90509392505050565b60606116978484600085611781565b90509392505050565b600080600090506000608084901c11156116c257608083901c92506080810190505b6000604084901c11156116dd57604083901c92506040810190505b6000602084901c11156116f857602083901c92506020810190505b6000601084901c111561171357601083901c92506010810190505b6000600884901c111561172e57600883901c92506008810190505b6000600484901c111561174957600483901c92506004810190505b6000600284901c111561176457600283901c92506002810190505b6000600184901c1115611778576001810190505b80915050919050565b6060824710156117c6576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016117bd906125cf565b60405180910390fd5b6000808673ffffffffffffffffffffffffffffffffffffffff1685876040516117ef919061262b565b60006040518083038185875af1925050503d806000811461182c576040519150601f19603f3d011682016040523d82523d6000602084013e611831565b606091505b50915091506118428783838761184e565b92505050949350505050565b606083156118b05760008351036118a857611868856118c3565b6118a7576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161189e9061268e565b60405180910390fd5b5b8290506118bb565b6118ba83836118e6565b5b949350505050565b6000808273ffffffffffffffffffffffffffffffffffffffff163b119050919050565b6000825111156118f95781518083602001fd5b806040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161192d91906126f2565b60405180910390fd5b600080fd5b600080fd5b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b600061196b82611940565b9050919050565b61197b81611960565b811461198657600080fd5b50565b60008135905061199881611972565b92915050565b6000819050919050565b6119b18161199e565b81146119bc57600080fd5b50565b6000813590506119ce816119a8565b92915050565b60008115159050919050565b6119e9816119d4565b81146119f457600080fd5b50565b600081359050611a06816119e0565b92915050565b600080fd5b600080fd5b600080fd5b60008083601f840112611a3157611a30611a0c565b5b8235905067ffffffffffffffff811115611a4e57611a4d611a11565b5b602083019150836020820283011115611a6a57611a69611a16565b5b9250929050565b6000819050919050565b611a8481611a71565b8114611a8f57600080fd5b50565b600081359050611aa181611a7b565b92915050565b60008060008060008060008060e0898b031215611ac757611ac6611936565b5b6000611ad58b828c01611989565b9850506020611ae68b828c016119bf565b9750506040611af78b828c016119bf565b9650506060611b088b828c016119f7565b955050608089013567ffffffffffffffff811115611b2957611b2861193b565b5b611b358b828c01611a1b565b945094505060a0611b488b828c016119bf565b92505060c0611b598b828c01611a92565b9150509295985092959890939650565b611b728161199e565b82525050565b6000604082019050611b8d6000830185611b69565b611b9a6020830184611b69565b9392505050565b611baa81611960565b82525050565b6000602082019050611bc56000830184611ba1565b92915050565b6000611bd682611960565b9050919050565b611be681611bcb565b8114611bf157600080fd5b50565b600081359050611c0381611bdd565b92915050565b60008060408385031215611c2057611c1f611936565b5b6000611c2e85828601611bf4565b9250506020611c3f85828601611bf4565b9150509250929050565b6000602082019050611c5e6000830184611b69565b92915050565b6000819050919050565b6000611c89611c84611c7f84611940565b611c64565b611940565b9050919050565b6000611c9b82611c6e565b9050919050565b6000611cad82611c90565b9050919050565b611cbd81611ca2565b82525050565b6000602082019050611cd86000830184611cb4565b92915050565b6000608082019050611cf36000830187611b69565b611d006020830186611b69565b611d0d6040830185611b69565b611d1a6060830184611b69565b95945050505050565b600060208284031215611d3957611d38611936565b5b6000611d4784828501611989565b91505092915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000611d8a8261199e565b9150611d958361199e565b9250828203905081811115611dad57611dac611d50565b5b92915050565b6000611dbe8261199e565b9150611dc98361199e565b9250828201905080821115611de157611de0611d50565b5b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b600060208284031215611e2c57611e2b611936565b5b6000611e3a84828501611bf4565b91505092915050565b600081519050611e52816119a8565b92915050565b600060208284031215611e6e57611e6d611936565b5b6000611e7c84828501611e43565b91505092915050565b600080fd5b600080fd5b60008085851115611ea357611ea2611e85565b5b83861115611eb457611eb3611e8a565b5b6020850283019150848603905094509492505050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b600081519050919050565b600082825260208201905092915050565b60005b83811015611f33578082015181840152602081019050611f18565b60008484015250505050565b6000601f19601f8301169050919050565b6000611f5b82611ef9565b611f658185611f04565b9350611f75818560208601611f15565b611f7e81611f3f565b840191505092915050565b6000608082019050611f9e6000830187611b69565b611fab6020830186611b69565b611fb86040830185611ba1565b8181036060830152611fca8184611f50565b905095945050505050565b600082825260208201905092915050565b7f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160008201527f6464726573730000000000000000000000000000000000000000000000000000602082015250565b6000612042602683611fd5565b915061204d82611fe6565b604082019050919050565b6000602082019050818103600083015261207181612035565b9050919050565b60006120838261199e565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff82036120b5576120b4611d50565b5b600182019050919050565b60006120cb8261199e565b9150600082036120de576120dd611d50565b5b600182039050919050565b7f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572600082015250565b600061211f602083611fd5565b915061212a826120e9565b602082019050919050565b6000602082019050818103600083015261214e81612112565b9050919050565b60006dffffffffffffffffffffffffffff82169050919050565b61217881612155565b811461218357600080fd5b50565b6000815190506121958161216f565b92915050565b600063ffffffff82169050919050565b6121b48161219b565b81146121bf57600080fd5b50565b6000815190506121d1816121ab565b92915050565b6000806000606084860312156121f0576121ef611936565b5b60006121fe86828701612186565b935050602061220f86828701612186565b9250506040612220868287016121c2565b9150509250925092565b60006122358261199e565b91506122408361199e565b925082820261224e8161199e565b9150828204841483151761226557612264611d50565b5b5092915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601260045260246000fd5b60006122a68261199e565b91506122b18361199e565b9250826122c1576122c061226c565b5b828204905092915050565b60006040820190506122e16000830185611ba1565b6122ee6020830184611b69565b9392505050565b600060608201905061230a6000830186611ba1565b6123176020830185611ba1565b6123246040830184611b69565b949350505050565b60008151905061233b816119e0565b92915050565b60006020828403121561235757612356611936565b5b60006123658482850161232c565b91505092915050565b7f5361666545524332303a204552433230206f7065726174696f6e20646964206e60008201527f6f74207375636365656400000000000000000000000000000000000000000000602082015250565b60006123ca602a83611fd5565b91506123d58261236e565b604082019050919050565b600060208201905081810360008301526123f9816123bd565b9050919050565b600061240b82611c90565b9050919050565b60008160601b9050919050565b600061242a82612412565b9050919050565b600061243c8261241f565b9050919050565b61245461244f82612400565b612431565b82525050565b60006124668285612443565b6014820191506124768284612443565b6014820191508190509392505050565b600081905092915050565b7fff00000000000000000000000000000000000000000000000000000000000000600082015250565b60006124c7600183612486565b91506124d282612491565b600182019050919050565b6124ee6124e982611ca2565b612431565b82525050565b6000819050919050565b61250f61250a82611a71565b6124f4565b82525050565b6000612520826124ba565b915061252c82866124dd565b60148201915061253c82856124fe565b60208201915061254c82846124fe565b602082019150819050949350505050565b7f416464726573733a20696e73756666696369656e742062616c616e636520666f60008201527f722063616c6c0000000000000000000000000000000000000000000000000000602082015250565b60006125b9602683611fd5565b91506125c48261255d565b604082019050919050565b600060208201905081810360008301526125e8816125ac565b9050919050565b600081905092915050565b600061260582611ef9565b61260f81856125ef565b935061261f818560208601611f15565b80840191505092915050565b600061263782846125fa565b915081905092915050565b7f416464726573733a2063616c6c20746f206e6f6e2d636f6e7472616374000000600082015250565b6000612678601d83611fd5565b915061268382612642565b602082019050919050565b600060208201905081810360008301526126a78161266b565b9050919050565b600081519050919050565b60006126c4826126ae565b6126ce8185611fd5565b93506126de818560208601611f15565b6126e781611f3f565b840191505092915050565b6000602082019050818103600083015261270c81846126b9565b90509291505056fea2646970667358221220e94abbff9707528f4c87367f309f3e0171e149951e866c0485d2c36b18a01b9264736f6c63430008130033" . parse () . expect ("invalid bytecode")
        });
    pub struct PancakeToaster<M>(ethers::contract::Contract<M>);
    impl<M> Clone for PancakeToaster<M> {
        fn clone(&self) -> Self {
            PancakeToaster(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for PancakeToaster<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for PancakeToaster<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(PancakeToaster))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> PancakeToaster<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), PANCAKETOASTER_ABI.clone(), client)
                .into()
        }
        #[doc = r" Constructs the general purpose `Deployer` instance based on the provided constructor arguments and sends it."]
        #[doc = r" Returns a new instance of a deployer that returns an instance of this contract after sending the transaction"]
        #[doc = r""]
        #[doc = r" Notes:"]
        #[doc = r" 1. If there are no constructor arguments, you should pass `()` as the argument."]
        #[doc = r" 1. The default poll duration is 7 seconds."]
        #[doc = r" 1. The default number of confirmations is 1 block."]
        #[doc = r""]
        #[doc = r""]
        #[doc = r" # Example"]
        #[doc = r""]
        #[doc = r" Generate contract bindings with `abigen!` and deploy a new contract instance."]
        #[doc = r""]
        #[doc = r" *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact."]
        #[doc = r""]
        #[doc = r" ```ignore"]
        #[doc = r" # async fn deploy<M: ethers::providers::Middleware>(client: ::std::sync::Arc<M>) {"]
        #[doc = r#"     abigen!(Greeter,"../greeter.json");"#]
        #[doc = r""]
        #[doc = r#"    let greeter_contract = Greeter::deploy(client, "Hello world!".to_string()).unwrap().send().await.unwrap();"#]
        #[doc = r"    let msg = greeter_contract.greet().call().await.unwrap();"]
        #[doc = r" # }"]
        #[doc = r" ```"]
        pub fn deploy<T: ethers::core::abi::Tokenize>(
            client: ::std::sync::Arc<M>,
            constructor_args: T,
        ) -> ::std::result::Result<
            ethers::contract::builders::ContractDeployer<M, Self>,
            ethers::contract::ContractError<M>,
        > {
            let factory = ethers::contract::ContractFactory::new(
                PANCAKETOASTER_ABI.clone(),
                PANCAKETOASTER_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `backRunSwapAll` (0xaadaa097) function"]
        pub fn back_run_swap_all(
            &self,
            token_in: ethers::core::types::Address,
            token_out: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([170, 218, 160, 151], (token_in, token_out))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `factory` (0xc45a0155) function"]
        pub fn factory(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([196, 90, 1, 85], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `frontRunSwap` (0x31d0baf4) function"]
        pub fn front_run_swap(
            &self,
            from: ethers::core::types::Address,
            amount_in: ethers::core::types::U256,
            amount_out: ethers::core::types::U256,
            eth_in: bool,
            path: ::std::vec::Vec<ethers::core::types::Address>,
            index_in: ethers::core::types::U256,
            parent_block_hash: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<
            M,
            (ethers::core::types::U256, ethers::core::types::U256),
        > {
            self.0
                .method_hash(
                    [49, 208, 186, 244],
                    (
                        from,
                        amount_in,
                        amount_out,
                        eth_in,
                        path,
                        index_in,
                        parent_block_hash,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `frontRunSwapExt` (0xc66e13cc) function"]
        pub fn front_run_swap_ext(
            &self,
            from: ethers::core::types::Address,
            amount_in: ethers::core::types::U256,
            amount_out: ethers::core::types::U256,
            eth_in: bool,
            path: ::std::vec::Vec<ethers::core::types::Address>,
            index_in: ethers::core::types::U256,
            parent_block_hash: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<
            M,
            (
                ethers::core::types::U256,
                ethers::core::types::U256,
                ethers::core::types::U256,
                ethers::core::types::U256,
            ),
        > {
            self.0
                .method_hash(
                    [198, 110, 19, 204],
                    (
                        from,
                        amount_in,
                        amount_out,
                        eth_in,
                        path,
                        index_in,
                        parent_block_hash,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `owner` (0x8da5cb5b) function"]
        pub fn owner(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([141, 165, 203, 91], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `renounceOwnership` (0x715018a6) function"]
        pub fn renounce_ownership(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([113, 80, 24, 166], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transferOwnership` (0xf2fde38b) function"]
        pub fn transfer_ownership(
            &self,
            new_owner: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([242, 253, 227, 139], new_owner)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `OwnershipTransferred` event"]
        pub fn ownership_transferred_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, OwnershipTransferredFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, OwnershipTransferredFilter> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for PancakeToaster<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Custom Error type `InsufficientBalance` with signature `InsufficientBalance(address)` and selector `[137, 127, 108, 88]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "InsufficientBalance", abi = "InsufficientBalance(address)")]
    pub struct InsufficientBalance {
        pub account: ethers::core::types::Address,
    }
    #[doc = "Custom Error type `InsufficientInputAmount` with signature `InsufficientInputAmount()` and selector `[9, 143, 181, 97]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "InsufficientInputAmount", abi = "InsufficientInputAmount()")]
    pub struct InsufficientInputAmount;
    #[doc = "Custom Error type `InsufficientLiquidity` with signature `InsufficientLiquidity()` and selector `[187, 85, 253, 39]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "InsufficientLiquidity", abi = "InsufficientLiquidity()")]
    pub struct InsufficientLiquidity;
    #[doc = "Custom Error type `InsufficientOutputAmount` with signature `InsufficientOutputAmount()` and selector `[66, 48, 28, 35]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "InsufficientOutputAmount", abi = "InsufficientOutputAmount()")]
    pub struct InsufficientOutputAmount;
    #[doc = "Custom Error type `InsufficientTokenBalance` with signature `InsufficientTokenBalance(address,address)` and selector `[186, 22, 5, 250]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(
        name = "InsufficientTokenBalance",
        abi = "InsufficientTokenBalance(address,address)"
    )]
    pub struct InsufficientTokenBalance {
        pub token: ethers::core::types::Address,
        pub account: ethers::core::types::Address,
    }
    #[doc = "Custom Error type `InvalidPath` with signature `InvalidPath()` and selector `[32, 219, 130, 103]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "InvalidPath", abi = "InvalidPath()")]
    pub struct InvalidPath;
    #[doc = "Custom Error type `SlippageExhausted` with signature `SlippageExhausted()` and selector `[202, 70, 144, 41]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "SlippageExhausted", abi = "SlippageExhausted()")]
    pub struct SlippageExhausted;
    #[doc = "Custom Error type `Uncled` with signature `Uncled()` and selector `[119, 151, 174, 109]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "Uncled", abi = "Uncled()")]
    pub struct Uncled;
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum PancakeToasterErrors {
        InsufficientBalance(InsufficientBalance),
        InsufficientInputAmount(InsufficientInputAmount),
        InsufficientLiquidity(InsufficientLiquidity),
        InsufficientOutputAmount(InsufficientOutputAmount),
        InsufficientTokenBalance(InsufficientTokenBalance),
        InvalidPath(InvalidPath),
        SlippageExhausted(SlippageExhausted),
        Uncled(Uncled),
    }
    impl ethers::core::abi::AbiDecode for PancakeToasterErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <InsufficientBalance as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterErrors::InsufficientBalance(decoded));
            }
            if let Ok(decoded) =
                <InsufficientInputAmount as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterErrors::InsufficientInputAmount(decoded));
            }
            if let Ok(decoded) =
                <InsufficientLiquidity as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterErrors::InsufficientLiquidity(decoded));
            }
            if let Ok(decoded) =
                <InsufficientOutputAmount as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterErrors::InsufficientOutputAmount(decoded));
            }
            if let Ok(decoded) =
                <InsufficientTokenBalance as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterErrors::InsufficientTokenBalance(decoded));
            }
            if let Ok(decoded) =
                <InvalidPath as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterErrors::InvalidPath(decoded));
            }
            if let Ok(decoded) =
                <SlippageExhausted as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterErrors::SlippageExhausted(decoded));
            }
            if let Ok(decoded) = <Uncled as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(PancakeToasterErrors::Uncled(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for PancakeToasterErrors {
        fn encode(self) -> Vec<u8> {
            match self {
                PancakeToasterErrors::InsufficientBalance(element) => element.encode(),
                PancakeToasterErrors::InsufficientInputAmount(element) => element.encode(),
                PancakeToasterErrors::InsufficientLiquidity(element) => element.encode(),
                PancakeToasterErrors::InsufficientOutputAmount(element) => element.encode(),
                PancakeToasterErrors::InsufficientTokenBalance(element) => element.encode(),
                PancakeToasterErrors::InvalidPath(element) => element.encode(),
                PancakeToasterErrors::SlippageExhausted(element) => element.encode(),
                PancakeToasterErrors::Uncled(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for PancakeToasterErrors {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                PancakeToasterErrors::InsufficientBalance(element) => element.fmt(f),
                PancakeToasterErrors::InsufficientInputAmount(element) => element.fmt(f),
                PancakeToasterErrors::InsufficientLiquidity(element) => element.fmt(f),
                PancakeToasterErrors::InsufficientOutputAmount(element) => element.fmt(f),
                PancakeToasterErrors::InsufficientTokenBalance(element) => element.fmt(f),
                PancakeToasterErrors::InvalidPath(element) => element.fmt(f),
                PancakeToasterErrors::SlippageExhausted(element) => element.fmt(f),
                PancakeToasterErrors::Uncled(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<InsufficientBalance> for PancakeToasterErrors {
        fn from(var: InsufficientBalance) -> Self {
            PancakeToasterErrors::InsufficientBalance(var)
        }
    }
    impl ::std::convert::From<InsufficientInputAmount> for PancakeToasterErrors {
        fn from(var: InsufficientInputAmount) -> Self {
            PancakeToasterErrors::InsufficientInputAmount(var)
        }
    }
    impl ::std::convert::From<InsufficientLiquidity> for PancakeToasterErrors {
        fn from(var: InsufficientLiquidity) -> Self {
            PancakeToasterErrors::InsufficientLiquidity(var)
        }
    }
    impl ::std::convert::From<InsufficientOutputAmount> for PancakeToasterErrors {
        fn from(var: InsufficientOutputAmount) -> Self {
            PancakeToasterErrors::InsufficientOutputAmount(var)
        }
    }
    impl ::std::convert::From<InsufficientTokenBalance> for PancakeToasterErrors {
        fn from(var: InsufficientTokenBalance) -> Self {
            PancakeToasterErrors::InsufficientTokenBalance(var)
        }
    }
    impl ::std::convert::From<InvalidPath> for PancakeToasterErrors {
        fn from(var: InvalidPath) -> Self {
            PancakeToasterErrors::InvalidPath(var)
        }
    }
    impl ::std::convert::From<SlippageExhausted> for PancakeToasterErrors {
        fn from(var: SlippageExhausted) -> Self {
            PancakeToasterErrors::SlippageExhausted(var)
        }
    }
    impl ::std::convert::From<Uncled> for PancakeToasterErrors {
        fn from(var: Uncled) -> Self {
            PancakeToasterErrors::Uncled(var)
        }
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(
        name = "OwnershipTransferred",
        abi = "OwnershipTransferred(address,address)"
    )]
    pub struct OwnershipTransferredFilter {
        #[ethevent(indexed)]
        pub previous_owner: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub new_owner: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `backRunSwapAll` function with signature `backRunSwapAll(address,address)` and selector `[170, 218, 160, 151]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "backRunSwapAll", abi = "backRunSwapAll(address,address)")]
    pub struct BackRunSwapAllCall {
        pub token_in: ethers::core::types::Address,
        pub token_out: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `factory` function with signature `factory()` and selector `[196, 90, 1, 85]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "factory", abi = "factory()")]
    pub struct FactoryCall;
    #[doc = "Container type for all input parameters for the `frontRunSwap` function with signature `frontRunSwap(address,uint256,uint256,bool,address[],uint256,bytes32)` and selector `[49, 208, 186, 244]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "frontRunSwap",
        abi = "frontRunSwap(address,uint256,uint256,bool,address[],uint256,bytes32)"
    )]
    pub struct FrontRunSwapCall {
        pub from: ethers::core::types::Address,
        pub amount_in: ethers::core::types::U256,
        pub amount_out: ethers::core::types::U256,
        pub eth_in: bool,
        pub path: ::std::vec::Vec<ethers::core::types::Address>,
        pub index_in: ethers::core::types::U256,
        pub parent_block_hash: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `frontRunSwapExt` function with signature `frontRunSwapExt(address,uint256,uint256,bool,address[],uint256,bytes32)` and selector `[198, 110, 19, 204]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "frontRunSwapExt",
        abi = "frontRunSwapExt(address,uint256,uint256,bool,address[],uint256,bytes32)"
    )]
    pub struct FrontRunSwapExtCall {
        pub from: ethers::core::types::Address,
        pub amount_in: ethers::core::types::U256,
        pub amount_out: ethers::core::types::U256,
        pub eth_in: bool,
        pub path: ::std::vec::Vec<ethers::core::types::Address>,
        pub index_in: ethers::core::types::U256,
        pub parent_block_hash: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `owner` function with signature `owner()` and selector `[141, 165, 203, 91]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "owner", abi = "owner()")]
    pub struct OwnerCall;
    #[doc = "Container type for all input parameters for the `renounceOwnership` function with signature `renounceOwnership()` and selector `[113, 80, 24, 166]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "renounceOwnership", abi = "renounceOwnership()")]
    pub struct RenounceOwnershipCall;
    #[doc = "Container type for all input parameters for the `transferOwnership` function with signature `transferOwnership(address)` and selector `[242, 253, 227, 139]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "transferOwnership", abi = "transferOwnership(address)")]
    pub struct TransferOwnershipCall {
        pub new_owner: ethers::core::types::Address,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum PancakeToasterCalls {
        BackRunSwapAll(BackRunSwapAllCall),
        Factory(FactoryCall),
        FrontRunSwap(FrontRunSwapCall),
        FrontRunSwapExt(FrontRunSwapExtCall),
        Owner(OwnerCall),
        RenounceOwnership(RenounceOwnershipCall),
        TransferOwnership(TransferOwnershipCall),
    }
    impl ethers::core::abi::AbiDecode for PancakeToasterCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <BackRunSwapAllCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterCalls::BackRunSwapAll(decoded));
            }
            if let Ok(decoded) =
                <FactoryCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterCalls::Factory(decoded));
            }
            if let Ok(decoded) =
                <FrontRunSwapCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterCalls::FrontRunSwap(decoded));
            }
            if let Ok(decoded) =
                <FrontRunSwapExtCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterCalls::FrontRunSwapExt(decoded));
            }
            if let Ok(decoded) = <OwnerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterCalls::Owner(decoded));
            }
            if let Ok(decoded) =
                <RenounceOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterCalls::RenounceOwnership(decoded));
            }
            if let Ok(decoded) =
                <TransferOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PancakeToasterCalls::TransferOwnership(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for PancakeToasterCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                PancakeToasterCalls::BackRunSwapAll(element) => element.encode(),
                PancakeToasterCalls::Factory(element) => element.encode(),
                PancakeToasterCalls::FrontRunSwap(element) => element.encode(),
                PancakeToasterCalls::FrontRunSwapExt(element) => element.encode(),
                PancakeToasterCalls::Owner(element) => element.encode(),
                PancakeToasterCalls::RenounceOwnership(element) => element.encode(),
                PancakeToasterCalls::TransferOwnership(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for PancakeToasterCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                PancakeToasterCalls::BackRunSwapAll(element) => element.fmt(f),
                PancakeToasterCalls::Factory(element) => element.fmt(f),
                PancakeToasterCalls::FrontRunSwap(element) => element.fmt(f),
                PancakeToasterCalls::FrontRunSwapExt(element) => element.fmt(f),
                PancakeToasterCalls::Owner(element) => element.fmt(f),
                PancakeToasterCalls::RenounceOwnership(element) => element.fmt(f),
                PancakeToasterCalls::TransferOwnership(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<BackRunSwapAllCall> for PancakeToasterCalls {
        fn from(var: BackRunSwapAllCall) -> Self {
            PancakeToasterCalls::BackRunSwapAll(var)
        }
    }
    impl ::std::convert::From<FactoryCall> for PancakeToasterCalls {
        fn from(var: FactoryCall) -> Self {
            PancakeToasterCalls::Factory(var)
        }
    }
    impl ::std::convert::From<FrontRunSwapCall> for PancakeToasterCalls {
        fn from(var: FrontRunSwapCall) -> Self {
            PancakeToasterCalls::FrontRunSwap(var)
        }
    }
    impl ::std::convert::From<FrontRunSwapExtCall> for PancakeToasterCalls {
        fn from(var: FrontRunSwapExtCall) -> Self {
            PancakeToasterCalls::FrontRunSwapExt(var)
        }
    }
    impl ::std::convert::From<OwnerCall> for PancakeToasterCalls {
        fn from(var: OwnerCall) -> Self {
            PancakeToasterCalls::Owner(var)
        }
    }
    impl ::std::convert::From<RenounceOwnershipCall> for PancakeToasterCalls {
        fn from(var: RenounceOwnershipCall) -> Self {
            PancakeToasterCalls::RenounceOwnership(var)
        }
    }
    impl ::std::convert::From<TransferOwnershipCall> for PancakeToasterCalls {
        fn from(var: TransferOwnershipCall) -> Self {
            PancakeToasterCalls::TransferOwnership(var)
        }
    }
    #[doc = "Container type for all return fields from the `backRunSwapAll` function with signature `backRunSwapAll(address,address)` and selector `[170, 218, 160, 151]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct BackRunSwapAllReturn {
        pub amount_out: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `factory` function with signature `factory()` and selector `[196, 90, 1, 85]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct FactoryReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `frontRunSwap` function with signature `frontRunSwap(address,uint256,uint256,bool,address[],uint256,bytes32)` and selector `[49, 208, 186, 244]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct FrontRunSwapReturn {
        pub our_amount_in: ethers::core::types::U256,
        pub our_amount_out: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `frontRunSwapExt` function with signature `frontRunSwapExt(address,uint256,uint256,bool,address[],uint256,bytes32)` and selector `[198, 110, 19, 204]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct FrontRunSwapExtReturn {
        pub our_amount_in: ethers::core::types::U256,
        pub our_amount_out: ethers::core::types::U256,
        pub new_reserve_in: ethers::core::types::U256,
        pub new_reserve_out: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `owner` function with signature `owner()` and selector `[141, 165, 203, 91]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct OwnerReturn(pub ethers::core::types::Address);
}
