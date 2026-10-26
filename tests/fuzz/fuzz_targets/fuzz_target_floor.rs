#![no_main]

use libfuzzer_sys::fuzz_target;
use tests::scenario::{ReachTally, Scenario};

// The production $5 floor, so liquidations seize dust remainders and covers write dust debt off.
fuzz_target!(|scenario: Scenario| scenario.run(500, &mut ReachTally::default()));
