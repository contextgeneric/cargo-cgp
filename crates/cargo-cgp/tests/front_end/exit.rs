//! Tests for the exit code the front-end propagates ([`cargo_cgp::exit::exit_code`]).

#[cfg(unix)]
mod unix {
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    use cargo_cgp::exit::exit_code;

    #[test]
    fn an_exit_code_passes_through() {
        // A wait status holds an exit code in its second byte.
        assert_eq!(exit_code(ExitStatus::from_raw(0)), 0);
        assert_eq!(exit_code(ExitStatus::from_raw(101 << 8)), 101);
    }

    #[test]
    fn a_signal_becomes_128_plus_its_number() {
        // A child ended by SIGINT (2) or SIGTERM (15) reports what a shell would: 130 and 143.
        assert_eq!(exit_code(ExitStatus::from_raw(2)), 130);
        assert_eq!(exit_code(ExitStatus::from_raw(15)), 143);
    }
}
