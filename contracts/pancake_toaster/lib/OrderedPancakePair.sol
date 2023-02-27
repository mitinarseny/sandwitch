import {IPancakePair} from "@pancake_swap/interfaces/IPancakePair.sol";

library OrderedPancakePair {
  struct Pair {
    IPancakePair _pair;
    bool _inverted;
  }
}
