use std::path::PathBuf;

use auth_rs::{card_details::extract_field_value, certs::VX_CUSTOM_CERT_FIELD_JURISDICTION};
use cacvote_server::client::Client;
use cacvote_server_client::{signer, AnySigner, PrivateKeySigner};
use clap::Parser;
use color_eyre::eyre::eyre;
use openssl::asn1::Asn1Time;
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use openssl::x509::{X509Builder, X509NameBuilder, X509};
use reqwest::Url;
use types_rs::cacvote::JurisdictionCode;
use uuid::Uuid;

#[derive(Parser)]
struct App {
    #[clap(flatten)]
    opts: GlobalOpts,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
struct GlobalOpts {
    #[clap(long)]
    since: Option<Uuid>,

    #[clap(
        long,
        env = "CACVOTE_SERVER_URL",
        default_value = "http://localhost:8000"
    )]
    cacvote_server_url: Url,

    #[clap(long, env = "SIGNING_CERT")]
    signing_cert: PathBuf,

    #[clap(long, env = "SIGNER")]
    signer: signer::Description,
}

impl GlobalOpts {
    fn signing_cert(&self) -> color_eyre::Result<X509> {
        let pem = std::fs::read(&self.signing_cert)?;
        let signing_cert = X509::from_pem(&pem)?;
        Ok(signing_cert)
    }

    fn jurisdiction_code(&self) -> color_eyre::Result<JurisdictionCode> {
        let signing_cert = self.signing_cert()?;
        let jurisdiction_code =
            extract_field_value(&signing_cert, VX_CUSTOM_CERT_FIELD_JURISDICTION)?
                .ok_or_else(|| eyre!("signing certificate does not have a jurisdiction code"))?;
        JurisdictionCode::try_from(jurisdiction_code)
            .map_err(|e| eyre!("invalid jurisdiction code: {e}"))
    }

    fn signer(&self) -> color_eyre::Result<AnySigner> {
        AnySigner::try_from(&self.signer)
    }

    fn client(&self) -> color_eyre::Result<Client> {
        let signing_cert = self.signing_cert()?;
        let signer = self.signer()?;
        Ok(Client::new(
            self.cacvote_server_url.clone(),
            signing_cert,
            signer,
        ))
    }
}

#[derive(Parser)]
enum Command {
    GetObject(GetObjectOpts),
    GetJournalEntries(GetJournalEntriesOpts),
    CreateObject(CreateObjectOpts),
}

#[derive(Parser)]
struct GetObjectOpts {
    object_id: Uuid,
}

#[derive(Parser)]
struct GetJournalEntriesOpts {
    #[clap(long)]
    since: Option<Uuid>,
}

#[derive(Parser)]
struct CreateObjectOpts {
    #[clap(skip)]
    card_keypair: Option<(X509, PKey<Private>)>,

    #[clap(short, long, default_value = "1234567890")]
    common_access_card_id: String,

    #[clap(short, long, default_value = "John")]
    given_name: String,

    #[clap(short, long, default_value = "Doe")]
    family_name: String,

    #[clap(short, long, default_value = "st.dev-jurisdiction")]
    jurisdiction_code: JurisdictionCode,
}

impl CreateObjectOpts {
    fn card_signer(&mut self) -> color_eyre::Result<AnySigner> {
        let (_, private_key) = self.get_or_create_card_keypair()?;
        Ok(Box::new(PrivateKeySigner::new(private_key.clone())))
    }

    fn card_cert(&mut self) -> color_eyre::Result<X509> {
        let (card_cert, _) = self.get_or_create_card_keypair()?;
        Ok(card_cert.clone())
    }

    /// Create a keypair and certificate for the common access card if one does
    /// not already exist.
    fn get_or_create_card_keypair(&mut self) -> color_eyre::Result<&(X509, PKey<Private>)> {
        let keypair = match self.card_keypair.take() {
            Some(keypair) => keypair,
            None => {
                let (certificate, private_key) = generate_keypair_and_certificate(
                    &self.common_access_card_id,
                    &self.given_name,
                    &self.family_name,
                )?;
                (certificate, private_key)
            }
        };

        self.card_keypair = Some(keypair);
        Ok(self.card_keypair.as_ref().expect("keypair should be set"))
    }
}

fn generate_keypair_and_certificate(
    common_access_card_id: &str,
    given_name: &str,
    family_name: &str,
) -> color_eyre::Result<(X509, PKey<Private>)> {
    // Generate RSA keypair
    let rsa = Rsa::generate(2048)?;
    let private_key = PKey::from_rsa(rsa)?;

    // Create X509 certificate
    let mut x509_builder = X509Builder::new()?;
    x509_builder.set_version(2)?;

    // Set subject name
    let mut name_builder = X509NameBuilder::new()?;
    name_builder.append_entry_by_nid(
        Nid::COMMONNAME,
        &format!(
            "{family_name}.{given_name}.{middle_name}.{common_access_card_id}",
            family_name = family_name,
            given_name = given_name,
            middle_name = "",
            common_access_card_id = common_access_card_id
        ),
    )?;
    let subject_name = name_builder.build();
    x509_builder.set_subject_name(&subject_name)?;

    // Set issuer name (same as subject for self-signed certificate)
    x509_builder.set_issuer_name(&subject_name)?;

    // Set validity period
    let not_before = Asn1Time::days_from_now(0)?;
    let not_after = Asn1Time::days_from_now(365)?;
    x509_builder.set_not_before(&not_before)?;
    x509_builder.set_not_after(&not_after)?;

    // Set public key
    x509_builder.set_pubkey(&private_key)?;

    // Sign the certificate with the private key
    x509_builder.sign(&private_key, MessageDigest::sha256())?;

    let certificate = x509_builder.build();

    Ok((certificate, private_key))
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv()?;

    let app = App::parse();

    match app.command {
        Command::GetObject(get_object_opts) => get_object(app.opts, get_object_opts).await?,
        Command::GetJournalEntries(get_journal_entries_opts) => {
            get_journal_entries(app.opts, get_journal_entries_opts).await?
        }
        Command::CreateObject(create_object_opts) => {
            create_object(app.opts, create_object_opts).await?
        }
    }

    Ok(())
}

async fn get_object(
    global_opts: GlobalOpts,
    get_object_opts: GetObjectOpts,
) -> color_eyre::Result<()> {
    let mut client = global_opts.client()?;

    let signed_object = match client.get_object_by_id(get_object_opts.object_id).await? {
        Some(signed_object) => signed_object,
        None => {
            println!("no object found with ID {}", get_object_opts.object_id);
            return Ok(());
        }
    };

    println!("signed object: {signed_object:#?}");

    let payload: types_rs::cacvote::Payload = serde_json::from_slice(&signed_object.payload)?;
    println!("object payload: {payload:#?}");

    Ok(())
}

async fn get_journal_entries(
    global_opts: GlobalOpts,
    get_journal_entries_opts: GetJournalEntriesOpts,
) -> color_eyre::Result<()> {
    let signing_cert = global_opts.signing_cert()?;
    let mut client = Client::new(
        global_opts.cacvote_server_url.clone(),
        signing_cert.clone(),
        global_opts.signer()?,
    );

    let entries = client
        .get_journal_entries(
            get_journal_entries_opts.since.as_ref(),
            Some(&global_opts.jurisdiction_code()?),
        )
        .await?;

    println!("entries: {entries:#?}");

    Ok(())
}

async fn create_object(
    global_opts: GlobalOpts,
    mut create_object_opts: CreateObjectOpts,
) -> color_eyre::Result<()> {
    let signer = create_object_opts.card_signer()?;
    let card_cert = create_object_opts.card_cert()?;

    let payload =
        types_rs::cacvote::Payload::RegistrationRequest(types_rs::cacvote::RegistrationRequest {
            common_access_card_id: create_object_opts.common_access_card_id,
            given_name: create_object_opts.given_name,
            family_name: create_object_opts.family_name,
            jurisdiction_code: create_object_opts.jurisdiction_code,
        });

    let payload = serde_json::to_vec(&payload)?;
    let signature = signer.sign(&payload)?;

    let signed_object = types_rs::cacvote::SignedObject {
        id: Uuid::new_v4(),
        election_id: None,
        payload,
        certificate: card_cert.to_pem()?,
        signature,
    };

    let mut client = global_opts.client()?;
    let object_id = client.create_object(signed_object).await?;
    println!("object_id: {object_id:?}");

    Ok(())
}
