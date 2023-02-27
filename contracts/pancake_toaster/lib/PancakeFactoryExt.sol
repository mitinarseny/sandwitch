pragma solidity ^0.8.18;

import {IERC20} from "@openzeppelin/token/ERC20/IERC20.sol";

import {IPancakeFactory} from "@pancake_swap/interfaces/IPancakeFactory.sol";
import {IPancakePair} from "@pancake_swap/interfaces/IPancakePair.sol";

library PancakeFactoryExt {
  struct Pair {
    IPancakePair pair;
    bool _inverted;
  }

  function PairFor(IPancakeFactory factory, )
}
