use std::{pin::Pin, sync::Arc, time::Duration};

use anyhow::Context;
use async_trait::async_trait;
use log::info;
use tokio::sync::mpsc::UnboundedSender;

const MB: f64 = 1024.0 * 1024.0;

pub trait DownloadReporter {
    type ReporterGuard: DownloadReporterGuard;

    fn start(&self, total: usize) -> anyhow::Result<Self::ReporterGuard>;
}

#[async_trait]
pub trait DownloadReporterGuard {
    async fn report(&self, chunk_size: usize) -> anyhow::Result<()>;
}

type ReportFunc = Arc<dyn Fn(f64, f64) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

pub struct DefaultDownloadReporter {
    report_interval: Duration,
    on_report: Option<ReportFunc>,
}

pub struct DefaultDownloadReporterGuard(UnboundedSender<usize>);

impl DefaultDownloadReporterGuard {
    fn new(total: f64, interval: Duration, on_report: Option<ReportFunc>) -> Self {
        let mut downloaded = 0f64;
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        info!("下载进度: {} MB/ {} MB ({:.2}%)",downloaded / MB, total/MB, downloaded / total * 100.0);
                        if let Some(ref f) = on_report {
                            f(downloaded,total).await;
                        }
                    }
                    msg = rx.recv() => {
                        let chunk = msg.unwrap_or(0);
                        if chunk == 0 {
                            return;
                        }
                        downloaded += chunk as f64;
                    }
                }
            }
        });
        Self(tx)
    }
}

#[async_trait]
impl DownloadReporterGuard for DefaultDownloadReporterGuard {
    async fn report(&self, chunk_size: usize) -> anyhow::Result<()> {
        self.0.send(chunk_size).context("send chunk size")
    }
}

impl DownloadReporter for DefaultDownloadReporter {
    type ReporterGuard = DefaultDownloadReporterGuard;

    fn start(&self, total: usize) -> anyhow::Result<Self::ReporterGuard> {
        Ok(DefaultDownloadReporterGuard::new(
            total as _,
            self.report_interval,
            self.on_report.clone(),
        ))
    }
}

impl DefaultDownloadReporter {
    pub fn new<F, Fut>(report_interval: Duration, on_report: Option<F>) -> Self
    where
        F: Fn(f64, f64) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self {
            report_interval,
            on_report: on_report.map(|f| {
                Arc::new(move |a, b| Box::pin(f(a, b)) as Pin<Box<dyn Future<Output = ()> + Send>>)
                    as _
            }),
        }
    }
}
