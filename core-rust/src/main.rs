use tma_core::{DeadlineGuard, MissionMachine, MissionState, accept_result};

fn self_test() -> Result<(), String> {
    let mut machine = MissionMachine::new();
    machine
        .transition(MissionState::Planned)
        .map_err(|e| e.to_string())?;
    machine
        .transition(MissionState::Running)
        .map_err(|e| e.to_string())?;
    machine
        .transition(MissionState::Validating)
        .map_err(|e| e.to_string())?;

    let guard = DeadlineGuard::new(500).map_err(str::to_owned)?;
    if !accept_result(&guard, 0.99, 0.90) {
        return Err("result gate rejected a valid result".to_string());
    }
    machine
        .transition(MissionState::Succeeded)
        .map_err(|e| e.to_string())?;

    println!(
        "tma-core status=ok state={:?} expired={}",
        machine.state(),
        guard.expired()
    );
    Ok(())
}

fn main() {
    let command = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "self-test".to_string());
    match command.as_str() {
        "self-test" => {
            if let Err(error) = self_test() {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("unknown command: {command}");
            std::process::exit(2);
        }
    }
}
