use crate::kubernetes;
use crate::ui;
use anyhow::Result;
use kube::config::Kubeconfig;
use std::env;
use std::path::PathBuf;

pub fn get_kubeconfig_path() -> Result<PathBuf> {
    match std::env::var("KUBECONFIG") {
        Ok(kubeconfig) => Ok(PathBuf::from(kubeconfig)),
        Err(_) => {
            let home = env::var("HOME").map_err(|_| anyhow::anyhow!("HOME env var is not set"))?;
            Ok(PathBuf::from(home).join(".kube/config"))
        }
    }
}

pub async fn select_context(
    mut config: Kubeconfig,
    kube_context: &Option<String>,
) -> Result<Kubeconfig> {
    if let Some(ctx) = kube_context {
        config.current_context = Some(ctx.clone());
        return Ok(config);
    }

    let current = &config.current_context;
    let input = config
        .contexts
        .iter()
        .map(|c| {
            if Some(&c.name) == current.as_ref() {
                format!("{} *", c.name)
            } else {
                c.name.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    match ui::handle_skim(input)? {
        Some(selection) => {
            let new_context = selection.trim_end_matches(" *").to_string();
            config.current_context = Some(new_context);
            Ok(config)
        }
        None => Err(anyhow::anyhow!("Selection cancelled")),
    }
}

pub async fn select_namespace(
    mut config: Kubeconfig,
    kube_namespace: &Option<String>,
) -> Result<Kubeconfig> {
    if let Some(ns) = kube_namespace {
        set_current_namespace(&mut config, ns.clone());
        return Ok(config);
    }

    let current_context = &config.current_context;
    let current_namespace = config
        .contexts
        .iter()
        .find(|ctx| Some(&ctx.name) == current_context.as_ref())
        .and_then(|ctx| ctx.context.as_ref())
        .and_then(|c| c.namespace.as_ref());
    let input = kubernetes::get_namespaces(current_namespace).await?;

    match ui::handle_skim(input)? {
        Some(new_namespace) => {
            let new_namespace = new_namespace.trim_end_matches(" *").to_string();
            if new_namespace.is_empty() {
                println!("No namespace selected");
                return Ok(config);
            }

            set_current_namespace(&mut config, new_namespace);
            Ok(config)
        }
        None => Err(anyhow::anyhow!("Selection cancelled")),
    }
}

fn set_current_namespace(config: &mut Kubeconfig, namespace: String) {
    if let Some(ctx) = config
        .contexts
        .iter_mut()
        .find(|c| Some(&c.name) == config.current_context.as_ref())
        && let Some(context) = &mut ctx.context
    {
        context.namespace = Some(namespace);
    }
}
