use serde::{Deserialize, Serialize};
use serde_json::{self};
use smol::channel::{Receiver, RecvError, SendError, Sender, TryRecvError, unbounded};
use std::fmt;
use std::{collections::HashMap, thread, time::Duration};
use ureq;

// Note: SERVER_ROOT constant is removed.

// --- Custom Error Type (No change needed) ---
#[derive(Debug)]
pub enum ApiError {
    UreqError(ureq::Error),
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    LogicError(String),
}
// ... (From implementations for ApiError remain the same) ...
impl From<ureq::Error> for ApiError {
    fn from(err: ureq::Error) -> Self {
        ApiError::UreqError(err)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::IoError(err)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::JsonError(err)
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::UreqError(e) => write!(f, "HTTP Request Error: {}", e),
            ApiError::IoError(e) => write!(f, "IO Error: {}", e),
            ApiError::JsonError(e) => write!(f, "JSON Deserialization Error: {}", e),
            ApiError::LogicError(s) => write!(f, "Logic Error: {}", s),
        }
    }
}

// --- Data Structures (No change needed) ---
#[derive(Debug, Clone, Serialize)] // Added Clone for config channel
pub struct WeightedItem {
    pub code: String,
    pub name: String,
    pub weight: f32, // weight is in kilo
    pub quantity: u32,
}
// ... (Workorders, WorkOrderLines, WorkOrderLine, Workorder, LoginResponse definitions remain the same) ...
#[derive(Debug, Deserialize)]
pub struct Workorders {
    pub value: Vec<Workorder>,
}
#[derive(Debug, Deserialize)]
pub struct WorkOrderLines {
    pub value: Vec<WorkOrderLine>,
}
#[derive(Debug, Deserialize)]
pub struct WorkOrderLine {
    #[serde(rename = "DocEntry")]
    pub doc_entry: u32,
    #[serde(rename = "LineNumber")]
    pub line_number: u32,
    #[serde(rename = "LineNumber2")]
    pub line_number2: u32,
    #[serde(rename = "SortId")]
    pub sort_id: u32,
    #[serde(rename = "Position")]
    pub position: String,
    #[serde(rename = "Barcode")]
    pub barcode: String,
    #[serde(rename = "ItemCode")]
    pub item_code: String,
    #[serde(rename = "ItemName")]
    pub item_name: String,
    #[serde(rename = "QuantityTotalWhsUnit")]
    pub quantity_total_whs_unit: f32,
}
#[derive(Debug, Deserialize)]
pub struct Workorder {
    #[serde(rename = "DocEntry")]
    pub doc_entry: u32,
    #[serde(rename = "DocNum")]
    pub doc_num: String,
    #[serde(rename = "DeliveryDate")]
    pub delivery_date: String,
    #[serde(rename = "CardCode")]
    pub card_code: String,
    #[serde(rename = "CardName")]
    pub card_name: String,
    #[serde(rename = "Lock")]
    pub lock: u32,
    #[serde(rename = "Closed")]
    pub closed: u32,
    #[serde(rename = "PriorityCode")]
    pub priority_code: String,
    #[serde(rename = "StartDate")]
    pub start_date: String,
    #[serde(rename = "EndDate")]
    pub end_date: String,
    #[serde(rename = "ItemCode")]
    pub item_code: String,
    #[serde(rename = "PlannedTime")]
    pub planned_time: f64,
    #[serde(rename = "WorkTime")]
    pub work_time: f64,
    #[serde(rename = "ReservedTime")]
    pub reserved_time: f64,
    #[serde(rename = "ProductionTypeId")]
    pub production_type_id: String,
    #[serde(rename = "Locked")]
    pub locked: bool,
    #[serde(rename = "ApsStatus")]
    pub aps_status: bool,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct LoginResponse {
    pub ret_code: u32,
    pub ret_text: String,
    #[serde(rename = "beas-sessionid")]
    pub beas_session_id: String,
}

pub fn article_weights() -> HashMap<&'static str, f64> {
    let mut m = HashMap::new();
    m.insert("ZURO-20160", 9.85);
    m.insert("ZURO-20161", 11.00);
    m.insert("ZURO-20162", 12.25);
    m.insert("ZURO-20163", 14.75);
    m.insert("ZURO-20164", 12.45);
    m.insert("ZURO-20165", 12.76);
    m.insert("ZURO-20166", 10.82);
    m.insert("ZURO-20167", 12.20);
    m.insert("ZURO-20168", 12.70);
    m.insert("ZURO-20169", 13.50);
    m.insert("ZURO-20170", 9.70);
    m.insert("ZURO-20171", 10.80);
    m.insert("ZURO-20183", 11.80);
    m.insert("ZURO-20188", 12.20);
    m.insert("ZURO-20190", 12.25);
    m.insert("ZURO-20200", 9.85);
    m.insert("ZURO-20209", 11.85);
    m.insert("ZURO-20210", 12.18);
    m
}

fn try_login(server_root: &str, pw: &str) -> Result<LoginResponse, ApiError> {
    let res = format!("{}Login", server_root);
    println!("trying login to: {}", res);
    let response: LoginResponse = ureq::post(&res)
        .send_json(ureq::json!({ "ServicePwd": pw }))
        .map_err(|e| ApiError::UreqError(e))?
        .into_json()
        .map_err(|e| ApiError::IoError(e))?;
    Ok(response)
}

fn get_work_orders(server_root: &str, session_id: &str) -> Result<Workorders, ApiError> {
    let res = format!(
        "{}{}",
        server_root,
        "Workorder?$select=DocEntry,DocNum,DeliveryDate,CardCode,CardName,Lock,Closed,PriorityCode,StartDate,EndDate,ItemCode,PlannedTime,WorkTime,ReservedTime,ProductionTypeId,Locked,ApsStatus&$filter=ApsStatus eq true and Closed eq 0&$orderby=StartDate asc"
    );
    println!("trying get workorders to: {}", res);
    let response = ureq::get(&res)
        .set("Cookie", session_id)
        .call()
        .map_err(|e| ApiError::UreqError(e))?;
    let body = response.into_string().map_err(|e| ApiError::IoError(e))?;
    let parsed: Workorders = serde_json::from_str(&body)?;
    Ok(parsed)
}

fn get_work_orders_bom(
    server_root: &str,
    session_id: &str,
    doc_entry: u32,
) -> Result<WorkOrderLines, ApiError> {
    let get_route_and_params = format!(
        "WorkorderBom?$filter=DocEntry eq {}&$select=DocEntry,LineNumber,LineNumber2,SortId,Position,Barcode,ItemCode,ItemName,QuantityTotalWhsUnit",
        doc_entry
    );
    let res = format!("{}{}", server_root, get_route_and_params);
    println!("trying get get_work_orders_bom to: {}", res);

    let response = ureq::get(&res)
        .set("Cookie", session_id)
        .call()
        .map_err(|e| ApiError::UreqError(e))?;
    let body = response.into_string().map_err(|e| ApiError::IoError(e))?;
    let parsed: WorkOrderLines = serde_json::from_str(&body)?;
    Ok(parsed)
}

fn get_weighted_item(
    item: &WorkOrderLine,
    weight_map: &HashMap<&'static str, f64>,
) -> Result<WeightedItem, ApiError> {
    let code: &str = &item.item_code;
    let weight = weight_map
        .get(code)
        .ok_or_else(|| ApiError::LogicError(format!("Missing weight for item code: {}", code)))?;
    let weighted_item = WeightedItem {
        code: item.item_code.clone(),
        name: item.item_name.clone(),
        weight: *weight as f32,
        quantity: item.quantity_total_whs_unit as u32,
    };
    Ok(weighted_item)
}

fn get_newest_order(work_orders: &Workorders) -> Result<&Workorder, ApiError> {
    work_orders
        .value
        .first()
        .ok_or_else(|| ApiError::LogicError("Workorders list is empty.".to_string()))
}

fn init_session(server_root: &str, pw: &str) -> Result<String, ApiError> {
    let session_id = try_login(server_root, pw)?.beas_session_id;
    Ok(format!("{}{};", "beas-sessionid=", session_id))
}

pub fn get_newest_weighted_item(
    server_root: &str,
    beas_session_id: &str,
) -> Result<WeightedItem, ApiError> {
    let article_weights = article_weights();
    let work_orders = get_work_orders(server_root, beas_session_id)?;
    let newest_order = get_newest_order(&work_orders)?;
    let bom = get_work_orders_bom(server_root, beas_session_id, newest_order.doc_entry)?;
    let found: Vec<_> = bom
        .value
        .iter()
        .filter(|item| item.item_code.contains("ZURO-"))
        .collect();
    let first_item = found
        .first()
        .ok_or_else(|| ApiError::LogicError("No ZURO- item found in BOM.".to_string()))?;
    let item = get_weighted_item(first_item, &article_weights)?;
    Ok(item)
}

// --- New Channel for Configuration ---
#[derive(Debug, Clone,Deserialize)]
pub struct ApiConfig {
    pub server_root: String,
    pub password: String,
    pub session_id: Option<String>,
}



type ConfigSender = Sender<ApiConfig>;
type ConfigReceiver = Receiver<ApiConfig>;



// A unified error type to handle both IO and JSON errors
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Io(err)
    }
}
impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::Json(err)
    }
}
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn read_api_config_from_file(file_path: &str) -> Result<ApiConfig, ConfigError> {
    // 1. Read the file content into a String
    let json_data = std::fs::read_to_string(file_path)?; // Uses ? to handle IoError

    // 2. Deserialize the JSON string into the ApiConfig struct
    let config: ApiConfig = serde_json::from_str(&json_data)?; // Uses ? to handle JsonError

    // 3. Return the successfully loaded config
    Ok(config)
}

#[derive(Debug)]
pub enum ChannelError {
    SendError(String),
    ReceiveError(String),
    Api(ApiError),
}

#[derive(Debug)]
pub enum WorkerError {
    Channel(ChannelError),
    WorkerStopped,
}
impl From<ChannelError> for WorkerError {
    fn from(err: ChannelError) -> Self {
        WorkerError::Channel(err)
    }
}

#[derive(Debug)]
pub enum CustomChannelError {
    Send(String),
    Receive(String),
}
impl From<RecvError> for CustomChannelError {
    fn from(err: RecvError) -> Self {
        CustomChannelError::Receive(format!("Blocking receive error: {}", err))
    }
}
impl From<SendError<WeightedItem>> for CustomChannelError {
    fn from(err: SendError<WeightedItem>) -> Self {
        CustomChannelError::Send(format!("Data send error: {}", err))
    }
}
impl From<SendError<ApiConfig>> for CustomChannelError {
    fn from(err: SendError<ApiConfig>) -> Self {
        CustomChannelError::Send(format!("Config send error: {}", err))
    }
}

impl From<smol::channel::TryRecvError> for ChannelError {
    fn from(err: smol::channel::TryRecvError) -> Self {
        ChannelError::ReceiveError(format!("TryRecv error: {}", err))
    }
}
// Ensure ChannelError handles blocking RecvError
impl From<RecvError> for ChannelError {
    fn from(err: RecvError) -> Self {
        ChannelError::ReceiveError(format!("Blocking Recv error: {}", err))
    }
}

// --- Channel Setup ---
type RequestSender = Sender<()>;
type RequestReceiver = Receiver<()>;

type ItemSender = Sender<WeightedItem>;
type ItemReceiver = Receiver<WeightedItem>;

pub struct WorkerChannels {
    pub config_tx: ConfigSender,
    pub request_tx: RequestSender,
    pub item_rx: ItemReceiver,
    request_rx: RequestReceiver,
    item_tx: ItemSender,
    config_rx: ConfigReceiver,
}

pub fn create_worker_channels() -> WorkerChannels {
    let (config_tx, config_rx) = unbounded::<ApiConfig>();
    let (request_tx, request_rx) = unbounded::<()>();
    let (item_tx, item_rx) = unbounded::<WeightedItem>();

    WorkerChannels {
        config_tx,
        request_tx,
        item_rx,
        request_rx,
        item_tx,
        config_rx,
    }
}

pub fn worker_sender_logic(
    request_rx: RequestReceiver,
    item_tx: ItemSender,
    config_rx: ConfigReceiver,
) -> Result<(), WorkerError> {
    let mut config: Option<ApiConfig> = None;

    // --- INITIAL CONFIGURATION STAGE ---
    // The thread loops and waits ONLY for the initial config before starting to poll for requests.
    while config.is_none() {
        match config_rx.try_recv() {
            Ok(new_config) => {
                config = Some(new_config);
                println!("{:?}", config);
                println!("[WORKER] ✅ Initial Config loaded. Starting request loop.");
                break; // Exit the while loop once configured
            }
            Err(TryRecvError::Empty) => {
                // Initial config hasn't arrived yet. Wait briefly.
                thread::sleep(Duration::from_millis(100));
            }
            Err(TryRecvError::Closed) => {
                eprintln!(
                    "[WORKER] ❌ Config channel closed before receiving initial config. Shutting down."
                );
                return Err(WorkerError::WorkerStopped);
            }
        }
    }

    let mut current_config = config.unwrap();
    let res = init_session(&current_config.server_root, &current_config.password);

    current_config.session_id = match res {
        Ok(beas_session_id) => Some(beas_session_id),
        Err(e) => {
            println!("{:?}", e);
            None
        }
    };
    println!("{:?}", current_config);

    // --- CONTINUOUS REQUEST/UPDATE POLLING STAGE ---
    loop {
        match request_rx.try_recv() {
            Ok(_) => {
                let result = get_newest_weighted_item(
                    &current_config.server_root,
                    &current_config.session_id.clone().unwrap(),
                );

                match result {
                    Ok(item) => {
                        println!("[WORKER] API success. Sending item: {}", item.code);
                        // Non-blocking send
                        if let Err(e) = item_tx.try_send(item) {
                            println!(
                                "[WORKER] ⚠️ Failed to send response (Receiver likely dropped): {}",
                                e
                            );
                        }
                    }
                    Err(e) => {
                        println!("[WORKER] ❌ API call failed: {}", e);
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                // No request is waiting. Sleep briefly to avoid high CPU usage.
                thread::sleep(Duration::from_millis(50));
            }
            Err(TryRecvError::Closed) => {
                eprintln!("[WORKER] ❌ Request channel closed. Shutting down.");
                return Err(WorkerError::WorkerStopped);
            }
        }
    }
}

// --- Client Logic (Simplified, using try_send/try_recv) ---

/// Sends a request signal and waits for the WeightedItem data response.
pub fn beas_client_receiver_logic(
    request_tx: RequestSender,
    item_rx: ItemReceiver,
) -> Result<WeightedItem, ChannelError> {
    println!("[CLIENT] Sending request signal...");

    // 1. Non-blocking Send the request signal
    request_tx
        .try_send(())
        .map_err(|e| ChannelError::SendError(format!("Request send error: {:?}", e)))?;

    println!("[CLIENT] Polling for WeightedItem data response...");

    // 2. Poll for the WeightedItem data response (Non-blocking receive, may take a few tries)
    // NOTE: In a real application, you might use a timeout or a different thread/async system
    // for the client to wait, but here we loop briefly to emulate the polling nature.
    for _i in 1..100 {
        // Try for up to 10 * 50ms = 500ms
        match item_rx.try_recv() {
            Ok(item) => {
                println!("[CLIENT] Data received.");
                return Ok(item);
            }
            Err(TryRecvError::Empty) => {
                // Wait briefly before trying again
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                // TryRecvError::Closed or other errors
                return Err(ChannelError::ReceiveError(format!(
                    "Client polling error: {}",
                    e
                )));
            }
        }

    }
    Err(ChannelError::ReceiveError(
        "Client response timeout.".to_string(),
    ))
}
pub fn start() -> (RequestSender, ItemReceiver, ConfigSender, thread::JoinHandle<Result<(), WorkerError>>) {
    let channels = create_worker_channels();    
    let consumer_request_tx = channels.request_tx.clone();
    let consumer_item_rx = channels.item_rx.clone();
    let consumer_config_tx = channels.config_tx.clone();

    // Start the Worker thread, passing the internal channel ends.
    let worker_handle = thread::spawn(move || {
        worker_sender_logic(channels.request_rx, channels.item_tx, channels.config_rx)
    });

    // Return the handles that the consumer will use.
    (
        consumer_request_tx,
        consumer_item_rx,
        consumer_config_tx,
        worker_handle,
    )
}

/*
pub fn main() {
    let (request_tx, item_rx, config_tx, _worker_handle) = start(); 
    let api_config = read_api_config_from_file("/tmp/api_config.json").unwrap();
    if let Err(e) = config_tx.try_send(api_config) {
        eprintln!("[MAIN] Failed to send new config: {:?}", e);
    }
    match beas_client_receiver_logic(request_tx.clone(), item_rx.clone()) {
        Ok(received_item) => {
            println!("[MAIN] ✅ T3 Success: Item code is {}", received_item.code);
        },
        Err(e) => eprintln!("[MAIN] ❌ T3 Failed: {:?}", e),
    }
    thread::sleep(Duration::from_millis(100)); // Give time for worker to process config
}
*/
