use tokio::sync::Mutex;
use crate::launch::ScreenOutputer::types::ScreenManager;
use crate::pairing::pairing_main::PairingSession;
use crate::launch::launching_main::LaunchingSession;
use std::sync::Arc;
use crate::launch::InputInjector::types::InputInjector;
use crate::launch::InputInjector::types::InputStat;
use crate::launch::ScreenOutputer::types::CaptureWayGeneral;

pub struct AppState {
  pub pairing: Arc<Mutex<Option<PairingSession>>>,
  pub launching: Arc<Mutex<Option<LaunchingSession>>>,
  pub input_trait: Arc<dyn InputInjector + Send + Sync>,
  pub input_stat: Arc<Mutex<Option<InputStat>>>,
  pub capture_trait: Arc<dyn CaptureWayGeneral + Send + Sync>,
  pub capture_stat: Arc<Mutex<Option<ScreenManager>>>, // TODO: LaunchingSessionに移す
}