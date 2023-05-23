extern crate rcgen;

pub mod security {
    use std::fs::File;
    use std::io::{Write};

    use mysql_common::time::{OffsetDateTime, Duration};
    use rcgen::{CertificateParams, Certificate, KeyPair};

    use crate::interface::server_api::UserCertificate;


    pub struct SecurityModule {
        ca_cert: &'static str,
        private_key: &'static str,
    }

    fn remove_sec_file(path: &std::path::Path, name: &str) {
        match std::fs::remove_file(path) {
            Ok(_) => (),
            Err(_) => {
                eprintln!("Cannot remove {name} file !");
            }
        }
    }

    impl SecurityModule {
        pub fn new(ca_path: &'static str, private_key: &'static str) -> Option<SecurityModule> {
            let cp = std::path::Path::new(ca_path);
            let pk = std::path::Path::new(private_key);
            if !cp.exists() || !pk.exists() {
                None
            } else {
                Some(SecurityModule {
                    ca_cert: ca_path,
                    private_key: private_key,
                })
            }
        }

        pub fn init_env(ca_path: &'static str, private_key: &'static str) -> bool {
            let cp = std::path::Path::new(ca_path);
            let pk = std::path::Path::new(private_key);
            if cp.exists() {
                remove_sec_file(cp, "ca_cert");
            }
            if pk.exists() {
                remove_sec_file(cp, "private key");
            }

            let subject = vec!["localhost".to_string(), "polydet.io".to_string()];
            let mut issuer = rcgen::DistinguishedName::new();
            issuer.push(rcgen::DnType::CommonName, "RustPolyDET");

            let mut certparams = CertificateParams::new(subject);
            certparams.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
            certparams.not_before = OffsetDateTime::now_utc();
            certparams.not_after = OffsetDateTime::now_utc() + Duration::days(365);
            certparams.distinguished_name = issuer;

            let cert = rcgen::Certificate::from_params(certparams);
            match cert {
                Ok(c) => {
                    let cafile = File::create(cp);
                    if cafile.is_err() {
                        eprintln!("Don't know why, but can't create file ! ");
                        return false;
                    }
                    let mut cafile = cafile.unwrap();
                    
                    if cafile.write_all(c.serialize_pem().unwrap().as_bytes()).is_err() {
                        eprintln!("Wait, so close !!!");
                        return false;
                    }

                    let pkfile = File::create(pk);
                    if pkfile.is_err() {
                        eprintln!("Don't know why, but can't save the pk !");
                        return false;
                    }
                    let mut pkfile = pkfile.unwrap();

                    if pkfile.write_all(c.serialize_private_key_pem().as_bytes()).is_err() {
                        eprintln!("Wait, whyyyyy !");
                        return false;
                    }
                    true
                }
                Err(_) => {
                    false
                }
            }
        }

        fn get_my_certificate(&self) -> Option<Certificate> {
            use std::fs;
            let certcontents = fs::read_to_string(self.ca_cert);
            let keycontents = fs::read_to_string(self.private_key);
            if certcontents.is_err() || keycontents.is_err() {
                return None;
            }
            let contents = certcontents.unwrap();
            let keycontents = keycontents.unwrap();
            let keypair = KeyPair::from_pem(&keycontents);
            if keypair.is_err() {
                eprintln!("{}", keypair.err().unwrap());
                return None;
            }
            let keypair = keypair.unwrap();
            let certparam = CertificateParams::from_ca_cert_pem(&contents, keypair);
            if certparam.is_err() {
                eprintln!("{}", certparam.err().unwrap());
                return None;
            }
            let cert = Certificate::from_params(certparam.unwrap());
            if cert.is_err() {
                eprintln!("{}", cert.err().unwrap());
                None
            } else {
                Some(cert.unwrap())
            }
        }

        pub fn generate_cert_for_user(&self, deviceid: &String, apphash: &String) -> Option<UserCertificate> {
            use base64::{Engine as _, engine::general_purpose};

            let subject = vec![deviceid.clone(), apphash.clone()];
            let mut issuer = rcgen::DistinguishedName::new();
            issuer.push(rcgen::DnType::CommonName, "RustPolyDET");

            let mut certparams = CertificateParams::new(subject);
            certparams.is_ca = rcgen::IsCa::ExplicitNoCa;
            certparams.not_before = OffsetDateTime::now_utc();
            certparams.not_after = OffsetDateTime::now_utc() + Duration::days(364 / 2);
            certparams.distinguished_name = issuer;

            let cert = rcgen::Certificate::from_params(certparams);
            let mycert = self.get_my_certificate();
            match cert {
                Ok(c) => {
                    if mycert.is_none() {
                        return None;
                    } 
                    let mycert = mycert.unwrap(); 
                    let res = c.serialize_pem_with_signer(&mycert);
                    if res.is_err() {
                        return None;
                    }
                    let pubk = c.get_key_pair().public_key_raw();
                    let priv_key = c.serialize_private_key_pem();
                    let pubk = general_purpose::STANDARD_NO_PAD.encode(pubk);

                    Some(UserCertificate {
                        device_id: deviceid.clone(),
                        app_hash: apphash.clone(),
                        public_key: pubk,
                        certificate: res.unwrap(),
                        private_key: priv_key
                    })
                }
                Err(_) => {
                    None
                }
            }
        }
    
    
    }
}
