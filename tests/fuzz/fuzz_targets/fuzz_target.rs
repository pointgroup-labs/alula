#![no_main]

use libfuzzer_sys::fuzz_target;
use tests::scenario::{ReachTally, Scenario};

fuzz_target!(|scenario: Scenario| scenario.run(0, &mut ReachTally::default()));
