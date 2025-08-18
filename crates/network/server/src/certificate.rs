use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, IsCa, Issuer, KeyPair,
    KeyUsagePurpose,
};
use std::fs::File;
use std::io::Write;

pub(super) fn create_certificate() -> (Certificate, Issuer<'static, KeyPair>) {
    // Schritt 1: KeyPair erzeugen (erstellt auch Public Key)
    let key_pair = KeyPair::generate().unwrap();
    let private_key_pem = key_pair.serialize_pem();

    // Private Key in Datei speichern
    let mut private_key_file = File::create("cert.key").expect("Failed to create private key file");
    private_key_file
        .write_all(private_key_pem.as_bytes())
        .expect("Failed to write private key to file");

    // Schritt 2: Params definieren
    let mut params = CertificateParams::new(vec!["localhost".to_string()])
        .expect("empty subject alt name can't produce error");
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params
        .distinguished_name
        .push(DnType::CommonName, "example.com");
    params
        .distinguished_name
        .push(DnType::OrganizationName, "MeineOrg");
    params.distinguished_name.push(DnType::CountryName, "DE");
    params
        .distinguished_name
        .push(DnType::LocalityName, "Wiesbaden");
    params
        .distinguished_name
        .push(DnType::StateOrProvinceName, "HE");
    params
        .distinguished_name
        .push(DnType::CountryName, "Germany");

    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params.key_usages.push(KeyUsagePurpose::KeyCertSign);
    params.key_usages.push(KeyUsagePurpose::CrlSign);

    // Optional: weitere Felder setzen (Serial, Gültigkeit, Extensions usw.)
    params.not_before = rcgen::date_time_ymd(2025, 8, 16);
    params.not_after = rcgen::date_time_ymd(2026, 8, 16);

    // Schritt 3: Zertifikat erstellen
    let cert = params.self_signed(&key_pair).unwrap();
    let cert_pem = cert.pem();

    // Zertifikat in Datei speichern
    let mut cert_file = File::create("cert.pem").expect("Failed to create certificate file");
    cert_file
        .write_all(cert_pem.as_bytes())
        .expect("Failed to write certificate to file");

    (cert, Issuer::new(params, key_pair))
}
