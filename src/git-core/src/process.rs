use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn configure_background_command(command: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
}

pub fn background_command(program: &str) -> Command {
    let mut command = Command::new(program);
    configure_background_command(&mut command);
    command
}

pub fn git_command() -> Command {
    background_command("git")
}
