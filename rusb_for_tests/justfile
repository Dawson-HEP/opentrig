# use PowerShell instead of sh on windows:
set windows-shell  := ["powershell.exe", "-c"]

rusb := "rusb_communication"
cli_file := "cli"
cli := if os()=="windows" { "./target/debug/cli.exe" } else { "./target/debug/cli" }

full_build:
   just build {{rusb}}
   just build {{cli_file}}

build file:
   cargo build --bin {{file}}

chamber *args:
   {{cli}} {{args}}