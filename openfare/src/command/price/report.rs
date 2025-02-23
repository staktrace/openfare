use anyhow::Result;

pub fn generate(
    package_locks: &openfare_lib::package::PackageLocks,
    config: &crate::common::config::Config,
) -> Result<Option<PriceReport>> {
    log::info!("Generating price report for package and it's dependencies.");

    // Handle primary package first.
    let mut package_reports = vec![];

    if let Some(primary_package) = &package_locks.primary_package {
        let primary_package_price_report = get_package_price_report(
            &primary_package,
            &package_locks.primary_package_lock,
            &config,
        )?;
        package_reports.push(primary_package_price_report);
    }

    for (package, package_lock) in &package_locks.dependencies_locks {
        let price_report = get_package_price_report(&package, &package_lock, &config)?;
        package_reports.push(price_report);
    }

    log::info!(
        "Number of price reports generated: {}",
        package_reports.len()
    );
    if package_reports.is_empty() {
        return Ok(None);
    }

    let total_price = package_reports
        .iter()
        .map(|r| r.price_quantity.unwrap_or(rust_decimal::Decimal::from(0)))
        .sum::<rust_decimal::Decimal>();

    let price_report = PriceReport {
        package_reports: package_reports,
        price: openfare_lib::lock::plan::price::Price {
            quantity: rust_decimal::Decimal::from(total_price),
            currency: config.core.preferred_currency.clone(),
        },
    };
    Ok(Some(price_report))
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct PriceReport {
    pub package_reports: Vec<PackagePriceReport>,
    pub price: openfare_lib::lock::plan::price::Price,
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct PackagePriceReport {
    pub package: openfare_lib::package::Package,
    pub price_quantity: Option<openfare_lib::lock::plan::price::Quantity>,
    pub notes: Vec<String>,
}

/// Given a package's OpenFare lock, create a corresponding price report.
pub fn get_package_price_report(
    package: &openfare_lib::package::Package,
    package_lock: &Option<openfare_lib::lock::Lock>,
    config: &crate::common::config::Config,
) -> Result<PackagePriceReport> {
    let package_lock = match package_lock {
        Some(c) => c,
        None => {
            return Ok(PackagePriceReport {
                package: package.clone(),
                price_quantity: None,
                notes: vec![],
            });
        }
    };

    let applicable_plans: Vec<_> = package_lock
        .plans
        .iter()
        .filter(|(_id, plan)| {
            plan.is_applicable(&config.metrics)
                .expect("plan applicable check")
                && plan.r#type == openfare_lib::lock::plan::PlanType::Compulsory
        })
        .collect();

    Ok(if let Some(preferred_plan) = applicable_plans.first() {
        PackagePriceReport {
            package: package.clone(),
            price_quantity: Some(if let Some(total) = &preferred_plan.1.payments.total {
                total.quantity
            } else {
                rust_decimal::Decimal::from(0)
            }),
            notes: vec![],
        }
    } else {
        PackagePriceReport {
            package: package.clone(),
            price_quantity: Some(rust_decimal::Decimal::from(0)),
            notes: vec![],
        }
    })
}
