use crate::common::config::FileStore;
use anyhow::{format_err, Result};
use openfare_lib::lock::payee::payment_methods::PaymentMethod;
use structopt::{self, StructOpt};

type Method = openfare_lib::lock::payee::payment_methods::BtcLightningKeysend;

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "no_version",
    no_version,
    global_settings = &[structopt::clap::AppSettings::DisableVersion]
)]
pub struct AddArguments {
    /// Public key destination
    #[structopt(name = "public-key")]
    pub public_key: String,
}

pub fn add(args: &AddArguments) -> Result<()> {
    let payment_method = Method::new(&args.public_key)?;
    let mut payees = crate::common::config::Payees::load()?;
    if let Some((_payee_name, payee)) = payees.active_mut()? {
        payee.set_payment_method(
            &(Box::new(payment_method)
                as Box<dyn openfare_lib::lock::payee::payment_methods::PaymentMethod>),
        )?;
        payees.dump()?;
    } else {
        return Err(format_err!("Failed to identify an active payee."));
    }
    Ok(())
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "no_version",
    no_version,
    global_settings = &[structopt::clap::AppSettings::DisableVersion]
)]
pub struct RemoveArguments {}

pub fn remove(_args: &RemoveArguments) -> Result<()> {
    let mut payees = crate::common::config::Payees::load()?;
    if let Some((_payee_name, payee)) = payees.active_mut()? {
        payee.remove_payment_method(&Method::associated_name())?;
        payees.dump()?;
    } else {
        return Err(format_err!("Failed to identify an active payee."));
    }
    Ok(())
}
