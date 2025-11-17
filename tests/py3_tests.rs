// use std::ffi::CStr;
// use pyo3::prelude::*;
// use pyo3::types::{IntoPyDict, PyModule, PyTuple};
// use pyo3::ffi::c_str;
//
//
// #[test]
// fn main() -> PyResult<()> {
//     Python::with_gil(|py| {
//         let sys = py.import("sys")?;
//         let version: String = sys.getattr("version")?.extract()?;
//
//         let locals = [("os", py.import("os")?)].into_py_dict(py)?;
//         let code = c_str!("os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'");
//         let user: String = py.eval(code, None, Some(&locals))?.extract()?;
//
//         let func: Bound<PyAny> = PyModule::from_code(
//             py,
//             c_str!(
//                 r#"import subprocess
// import sys
//
// def ensure_sympy_installed():
//     try:
//         import sympy
//         return True
//     except ImportError:
//         # subprocess.check_call([sys.executable, '-m', 'pip', 'install', 'sympy'])
//         return False"#),
//             c_str!(""),
//             c_str!(""),
//         )?.getattr("ensure_sympy_installed")?;
//         let success: bool = func.call0()?.extract()?;
//         let python_exec: String = sys.getattr("executable")?.extract()?;
//
//         if !success {
//             println!("Installing sympy... using {}", python_exec);
//             // Install sympy using rust
//             let out = std::process::Command::new(python_exec)
//                 .arg("-m")
//                 .arg("pip")
//                 .arg("install")
//                 .arg("sympy")
//                 .output()?;
//             let out = String::from_utf8_lossy(&out.stdout);
//             if out.contains("Successfully installed") {
//                 println!("Sympy installed successfully");
//             } else {
//                 println!("Failed to install sympy");
//             }
//         }
//
//         println!("Hello {}, I'm Python {}", user, version);
//         println!("Success: {}", success);
//         Ok(())
//     })
// }