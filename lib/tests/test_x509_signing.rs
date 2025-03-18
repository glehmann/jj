#[cfg(unix)]
use std::fs::Permissions;
use std::io::Write as _;
#[cfg(unix)]
use std::os::unix::prelude::PermissionsExt as _;
use std::process::Command;
use std::process::Stdio;

use assert_matches::assert_matches;
use insta::assert_debug_snapshot;
use jj_lib::signing::SigStatus;
use jj_lib::signing::SignError;
use jj_lib::signing::SigningBackend as _;
use jj_lib::x509_signing::X509Backend;
use testutils::ensure_running_outside_ci;

static FINGERPRINT: &str = "8F08B1D7DDEEFA91C25F8681F6494FF2E03DEADE";

static PRIVATE_KEY: &str = r#"Issuer ...: /CN=JJ/O=X509 Signing Test
Serial ...: 4C2B796B46814D39
Subject ..: /CN=JJ/O=X509 Signing Test
    aka ..: someone@example.com
Keygrip ..: E595CEA8D5E74F60DEDF96753E041F64157DFBF6

-----BEGIN PKCS12-----
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
fXcmHPrQS3ICAggA
-----END PKCS12-----
"#;

struct X509Environment {
    homedir: tempfile::TempDir,
}

impl X509Environment {
    fn new() -> Result<Self, std::process::Output> {
        let dir = tempfile::Builder::new()
            .prefix("jj-x509-signing-test-")
            .tempdir()
            .unwrap();

        let path = dir.path();

        #[cfg(unix)]
        std::fs::set_permissions(path, Permissions::from_mode(0o700)).unwrap();

        std::fs::write(path.join("trustlist.txt"), format!("{FINGERPRINT} S\n")).unwrap();

        let mut gpgsm = std::process::Command::new("gpgsm")
            .arg("--homedir")
            .arg(path)
            .arg("--batch")
            .arg("--pinentry-mode")
            .arg("loopback")
            .arg("--import")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        gpgsm
            .stdin
            .as_mut()
            .unwrap()
            .write_all(PRIVATE_KEY.as_bytes())
            .unwrap();

        gpgsm.stdin.as_mut().unwrap().flush().unwrap();

        let res = gpgsm.wait_with_output().unwrap();

        if !res.status.success() && res.status.code() != Some(2) {
            eprintln!("Failed to add certificate.");
            eprintln!("{}", String::from_utf8_lossy(&res.stderr));
            return Err(res);
        }

        Ok(X509Environment { homedir: dir })
    }
}

macro_rules! gpgsm_guard {
    () => {
        if Command::new("gpgsm").arg("--version").status().is_err() {
            ensure_running_outside_ci("`gpgsm` must be in the PATH");
            eprintln!("Skipping test because gpgsm is not installed on the system");
            return;
        }
    };
}

fn backend(env: &X509Environment) -> X509Backend {
    // don't really need faked time for current tests,
    // but probably will need it for end-to-end cli tests
    X509Backend::new("gpgsm".into(), false, "someone@example.com".to_owned()).with_extra_args(&[
        "--homedir".into(),
        env.homedir.path().as_os_str().into(),
        "--faked-system-time=1742324579!".into(),
    ])
}

#[test]
#[cfg_attr(windows, ignore = "stuck randomly on Windows CI #3140")] // FIXME
fn x509_signing_roundtrip() {
    gpgsm_guard!();

    let env = X509Environment::new().unwrap();
    let backend = backend(&env);
    let data = b"hello world";
    let signature = backend.sign(data, None);
    let signature = signature.unwrap();

    let check = backend.verify(data, &signature).unwrap();
    assert_eq!(check.status, SigStatus::Good);
    assert_eq!(
        check.key.unwrap(),
        "8F08B1D7DDEEFA91C25F8681F6494FF2E03DEADE"
    );
    assert_eq!(check.display.unwrap(), "/CN=JJ/O=X509 Signing Test");

    let check = backend.verify(b"so so bad", &signature).unwrap();
    assert_eq!(check.status, SigStatus::Bad);
    assert_eq!(
        check.key.unwrap(),
        "8F08B1D7DDEEFA91C25F8681F6494FF2E03DEADE"
    );
    assert_eq!(check.display.unwrap(), "/CN=JJ/O=X509 Signing Test");
}

#[test]
#[cfg_attr(windows, ignore = "stuck randomly on Windows CI #3140")] // FIXME
fn x509_signing_roundtrip_explicit_key() {
    gpgsm_guard!();

    let env = X509Environment::new().unwrap();
    let backend = backend(&env);
    let data = b"hello world";
    let signature = backend.sign(data, Some("someone@example.com")).unwrap();

    assert_debug_snapshot!(backend.verify(data, &signature).unwrap(), @r#"
    Verification {
        status: Good,
        key: Some(
            "8F08B1D7DDEEFA91C25F8681F6494FF2E03DEADE",
        ),
        display: Some(
            "/CN=JJ/O=X509 Signing Test",
        ),
    }
    "#);
    assert_debug_snapshot!(backend.verify(b"so so bad", &signature).unwrap(), @r#"
    Verification {
        status: Bad,
        key: Some(
            "8F08B1D7DDEEFA91C25F8681F6494FF2E03DEADE",
        ),
        display: Some(
            "/CN=JJ/O=X509 Signing Test",
        ),
    }
    "#);
}

#[test]
#[cfg_attr(windows, ignore = "stuck randomly on Windows CI #3140")] // FIXME
fn x509_unknown_key() {
    gpgsm_guard!();

    let env = X509Environment::new().unwrap();
    let backend = backend(&env);
    let signature = br"-----BEGIN SIGNED MESSAGE-----
    MIAGCSqGSIb3DQEHAqCAMIACAQExDzANBglghkgBZQMEAgEFADCABgkqhkiG9w0B
    BwEAADGCAnYwggJyAgEBMDUwKTEaMBgGA1UEChMRWDUwOSBTaWduaW5nIFRlc3Qx
    CzAJBgNVBAMTAkpKAgh8bds9GXiZmzANBglghkgBZQMEAgEFAKCBkzAYBgkqhkiG
    9w0BCQMxCwYJKoZIhvcNAQcBMBwGCSqGSIb3DQEJBTEPFw0yNTAzMTgyMDAzNDBa
    MCgGCSqGSIb3DQEJDzEbMBkwCwYJYIZIAWUDBAECMAoGCCqGSIb3DQMHMC8GCSqG
    SIb3DQEJBDEiBCCpSJBPLw9Hm4+Bl2lLMBhLDS7Rwc0qHsD7hdKZoZKkRzANBgkq
    hkiG9w0BAQEFAASCAYANOvWCJuOKn018s731TWFHq5wS13xB7L83/2q8Mi9cQ3YT
    kq8CQlyJV0spIW7dwztjsllX8X2szE4N0l83ghf3ol6B6n9Vyb844oKgb6cwc9uX
    S8D1yiaj1Mfft3PDp+THH+ESezw1Djzj7E53Yx5j3kna/ylJhheg3raWit2MUxI0
    V42Svm4PLcpOf+ywzstlSSx9p6Y8woctdkMkpyivNCsfwlRARFGSTP3G9DXZNv03
    WZ51zlMT8lsYbT9EJUxzXuEpcJZJL0TYcbJ3n7uSopivHk843onIc71gbH/ByuMp
    qokJ7jYzEMrk0YowzsD7wrtwhF5OgpW5ane8vuyquLOrRNX9H/TooE4+8OCM6nvQ
    w7jgv1/hsdtDnZCkVaM0plhb2btE7Awgol5M8f9IDz1Z+b0t4ydc/iqHtE9yaqvZ
    +aT9XXKKcj9XBhi1S790B4r8YoDyeiyzBs0gwvMuWjWMS7wixTbgx+IkQUrkgTLY
    xiNbRmGtEonl9d8JS/IAAAAAAAA=
    -----END SIGNED MESSAGE-----
    ";
    assert_matches!(
        backend.verify(b"hello world", signature),
        Err(SignError::InvalidSignatureFormat)
    );
    assert_matches!(
        backend.verify(b"so bad", signature),
        Err(SignError::InvalidSignatureFormat)
    );
}

#[test]
#[cfg_attr(windows, ignore = "stuck randomly on Windows CI #3140")] // FIXME
fn x509_invalid_signature() {
    gpgsm_guard!();

    let env = X509Environment::new().unwrap();
    let backend = backend(&env);
    let signature = br"-----BEGIN SIGNED MESSAGE-----
    super duper invalid
    -----END SIGNED MESSAGE-----";

    // Small data: gpgsm command will exit late.
    assert_matches!(
        backend.verify(b"a", signature),
        Err(SignError::InvalidSignatureFormat)
    );

    // Large data: gpgsm command will exit early because the signature is invalid.
    assert_matches!(
        backend.verify(&b"a".repeat(100 * 1024), signature),
        Err(SignError::InvalidSignatureFormat)
    );
}
