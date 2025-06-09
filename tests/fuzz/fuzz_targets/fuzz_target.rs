#![no_main]

use {
    libfuzzer_sys::fuzz_target,
    tests::{assert_invariants, Input, TestFixture},
};

fuzz_target!(|input: Input| {
    let fixture = TestFixture::new();

    fixture.e.cost_estimate().budget().reset_unlimited();

    let commands = input.commands;

    for command in commands {
        command.run(&fixture);
        assert_invariants(&fixture);
    }
});
