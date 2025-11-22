// MSSCS Mobile - Desktop entry point
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    msscs_mobile_lib::run();
}
