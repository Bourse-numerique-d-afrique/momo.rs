// MTN MoMo Callback Server - Library

use std::error::Error;

use futures_core::Stream;
use poem::listener::TcpListener;
use poem::middleware::AddData;
use poem::web::Data;
use poem::{handler, post, get, Body, Request, Response, Route, Server, EndpointExt};
use tokio::sync::mpsc::{self, Sender};
use tracing::{error, info, warn};

use mtnmomo::{CallbackResponse, MomoUpdates};

/// Configuration structure for the MTN MoMo callback server.
///
/// This structure holds all the necessary configuration parameters for running
/// the callback server, including network binding configuration and server settings.
///
/// ## Examples
///
/// ```rust,no_run
/// # use momo_callback_server::CallbackServerConfig;
///
/// // Create configuration with default values
/// let config = CallbackServerConfig::default();
///
/// // Create configuration with custom values
/// let config = CallbackServerConfig {
///     http_port: 8500,
///     host: "127.0.0.1".to_string(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct CallbackServerConfig {
    /// HTTP port number for the server to bind to.
    ///
    /// Default is 8500. For development deployments, use ports > 1024.
    pub http_port: u16,

    /// Host address to bind the server to.
    ///
    /// Default is "127.0.0.1" (localhost only). Use "0.0.0.0" to bind to all interfaces.
    pub host: String,
}

impl Default for CallbackServerConfig {
    /// Creates a default configuration for the callback server.
    ///
    /// This implementation provides sensible fallback defaults:
    ///
    /// - `http_port`: 8500 (custom HTTP port)
    /// - `host`: "127.0.0.1" (localhost only)
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use momo_callback_server::CallbackServerConfig;
    ///
    /// // Create default configuration
    /// let config = CallbackServerConfig::default();
    /// assert_eq!(config.http_port, 8500);
    /// ```
    fn default() -> Self {
        Self {
            http_port: 8500,
            host: "127.0.0.1".to_string(),
        }
    }
}

/// Health check endpoint handler.
///
/// This endpoint provides a simple health check for monitoring and load balancing.
/// It always returns "OK" with a 200 status code when the server is running.
///
/// ## Endpoint
///
/// - **URL**: `GET /health`
/// - **Response**: `200 OK` with body "OK"
///
/// ## Usage
///
/// This endpoint is commonly used by:
/// - Load balancers for health checks
/// - Monitoring systems for uptime verification
/// - Container orchestrators (Docker, Kubernetes) for readiness probes
///
/// ## Examples
///
/// ```bash
/// # Using curl
/// curl https://your-server.com/health
/// # Response: OK
///
/// # Using wget
/// wget -qO- https://your-server.com/health
/// # Response: OK
/// ```
///
/// ## Monitoring Integration
///
/// ```yaml
/// # Kubernetes readiness probe example
/// readinessProbe:
///   httpGet:
///     path: /health
///     port: 443
///     scheme: HTTPS
///   initialDelaySeconds: 10
///   periodSeconds: 5
/// ```
#[handler]
async fn health_check() -> &'static str {
    "OK"
}

/// Primary callback handler for all MTN MoMo callback requests.
///
/// This handler processes incoming callback requests from the MTN MoMo API,
/// parsing the JSON payload, validating the callback type, and forwarding
/// the processed callback to the appropriate business logic handlers.
///
/// ## Parameters
///
/// - `req`: The incoming HTTP request containing headers and metadata
/// - `body`: The request body containing the JSON callback payload
/// - `sender`: Channel sender for forwarding parsed callbacks to processors
///
/// ## Callback Flow
///
/// 1. **Request Reception**: Receives POST request from MTN MoMo API
/// 2. **Body Extraction**: Reads and converts request body to string
/// 3. **JSON Parsing**: Attempts to parse body as `CallbackResponse`
/// 4. **Channel Forwarding**: Sends parsed callback to processing channel
/// 5. **Response**: Returns success response to MTN MoMo API
///
/// ## Supported Callback Types
///
/// All MTN MoMo callback types are supported through the `CallbackResponse` enum.
///
/// ## Error Handling
///
/// - **JSON Parse Errors**: Logged as warnings, but don't fail the request
/// - **Channel Send Errors**: Logged as errors indicating internal issues
/// - **Network Errors**: Handled by the web framework
///
/// ## Response Format
///
/// Returns a JSON response indicating successful receipt:
/// ```json
/// {
///   "status": "success",
///   "message": "Callback received successfully"
/// }
/// ```
#[handler]
async fn mtn_callback_handler(
    req: &Request,
    mut body: Body,
    sender: Data<&Sender<MomoUpdates>>,
) -> Result<Response, poem::Error> {
    let remote_address = req.remote_addr().to_string();
    let body_string = body.into_string().await?;
    
    info!("Received callback from {}", remote_address);
    info!("Raw callback body: {}", body_string);
    
    let response_result: Result<CallbackResponse, serde_json::Error> =
        serde_json::from_str(&body_string);
    
    match response_result {
        Ok(callback_response) => {
            let momo_updates = MomoUpdates {
                remote_address,
                response: callback_response,
            };
            
            if let Err(e) = sender.send(momo_updates).await {
                error!("Failed to send callback update: {}", e);
            } else {
                info!("Successfully processed callback");
            }
        }
        Err(e) => {
            warn!("Failed to parse callback body: {} - Body: {}", e, body_string);
        }
    }
    
    Ok(Response::builder()
        .status(poem::http::StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(r#"{"status": "success", "message": "Callback received successfully"}"#))
}

/// Creates and configures all callback routes for MTN MoMo services.
///
/// This function sets up the complete routing table for the callback server,
/// mapping URL patterns to the callback handler function. Each route corresponds
/// to a specific MTN MoMo service and operation type.
///
/// ## Route Structure
///
/// All routes follow the pattern: `/{service}_{operation}`
///
/// Where:
/// - `service`: The MTN MoMo service (collection, disbursement, remittance)
/// - `operation`: The specific operation (request_to_pay, deposit, transfer, etc.)
///
/// ## Supported Routes
///
/// ### Collection Service
/// - `/collection_request_to_pay`: Payment requests
/// - `/collection_request_to_withdraw_v1`: Withdrawal v1
/// - `/collection_request_to_withdraw_v2`: Withdrawal v2
/// - `/collection_invoice`: Invoice operations
/// - `/collection_payment`: Payment operations
/// - `/collection_preapproval`: Pre-approval operations
///
/// ### Disbursement Service
/// - `/disbursement_deposit_v1`: Deposit v1 operations
/// - `/disbursement_deposit_v2`: Deposit v2 operations
/// - `/disbursement_refund_v1`: Refund v1 operations
/// - `/disbursement_refund_v2`: Refund v2 operations
/// - `/disbursement_transfer`: Transfer operations
///
/// ### Remittance Service
/// - `/remittance_cash_transfer`: Cash transfers
/// - `/remittance_transfer`: Regular transfers
///
/// ### Utility Routes
/// - `/health`: Health check endpoint (GET)
///
/// ## Middleware Stack
///
/// The routes are enhanced with the following middleware:
/// - **Tracing**: Request/response logging and tracing
/// - **CORS**: Cross-origin resource sharing support
/// - **Compression**: Response compression for efficiency
/// - **RequestId**: Unique request ID generation for tracking
/// - **Data**: Shared data injection for callback processing
///
/// ## Examples
///
/// ```rust,no_run
/// # use momo_callback_server::create_callback_routes;
/// use poem::{Route, Server};
///
/// // Create the routes
/// let routes = create_callback_routes();
///
/// // The routes can be used with the poem server
/// // Server::new(listener).run(routes).await;
/// ```
///
/// ## URL Examples
///
/// ```bash
/// # Collection payment callback
/// POST /collection_request_to_pay
///
/// # Disbursement deposit callback
/// POST /disbursement_deposit_v1
///
/// # Health check
/// GET /health
/// ```
pub fn create_callback_routes() -> Route {
    Route::new()
        .at(
            "/collection_request_to_pay",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/collection_request_to_withdraw_v1",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/collection_request_to_withdraw_v2",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/collection_invoice",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/collection_payment",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/collection_preapproval",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        // Disbursement callbacks
        .at(
            "/disbursement_deposit_v1",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/disbursement_deposit_v2",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/disbursement_refund_v1",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/disbursement_refund_v2",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/disbursement_transfer",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        // Remittance callbacks
        .at(
            "/remittance_cash_transfer",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/remittance_transfer",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        // Health check endpoint
        .at("/health", get(health_check))
}

/// Starts the MTN MoMo callback server with the specified configuration.
///
/// This is the main function that initializes and starts the callback server. It creates
/// the web application with all routes and middleware, starts the server in a background
/// task, and returns a stream of processed callbacks.
///
/// ## Parameters
///
/// - `config`: Server configuration including network binding settings
///
/// ## Returns
///
/// - `Ok(Stream<Item = MomoUpdates>)`: A stream of processed callback updates
/// - `Err(Box<dyn Error>)`: Startup error (network binding, etc.)
///
/// ## Server Lifecycle
///
/// 1. **Route Setup**: Creates all callback routes and middleware stack
/// 2. **Network Binding**: Binds to the specified host and port
/// 3. **Background Task**: Spawns server in a background tokio task
/// 4. **Channel Setup**: Creates communication channel for callback processing
/// 5. **Stream Return**: Returns async stream of incoming callbacks
///
/// ## Graceful Shutdown
///
/// The server supports graceful shutdown via Ctrl+C (SIGINT):
/// - Ongoing requests are allowed to complete
/// - New connections are rejected
/// - Server cleanly shuts down and logs completion
///
/// ## Error Scenarios
///
/// The function will return an error in these cases:
/// - Port is already in use or requires elevated privileges
/// - Network interface is not available
/// - Insufficient memory or system resources
///
/// ## Examples
///
/// ### Basic Usage
///
/// ```rust,no_run
/// use momo_callback_server::{CallbackServerConfig, start_callback_server};
/// use futures_util::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = CallbackServerConfig::default();
///     let mut callback_stream = start_callback_server(config).await?;
///
///     println!("Server started, processing callbacks...");
///
///     while let Some(callback) = callback_stream.next().await {
///         println!("Received callback: {:?}", callback.response);
///         // Process the callback according to your business logic
///     }
///
///     Ok(())
/// }
/// ```
///
/// ### Custom Configuration
///
/// ```rust,no_run
/// use momo_callback_server::{CallbackServerConfig, start_callback_server};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = CallbackServerConfig {
///         http_port: 8500,  // Custom port
///         host: "127.0.0.1".to_string(),  // Localhost only
///     };
///
///     let callback_stream = start_callback_server(config).await?;
///     // Handle callbacks...
///     Ok(())
/// }
/// ```
///
/// ### With Error Handling
///
/// ```rust,no_run
/// use momo_callback_server::{CallbackServerConfig, start_callback_server};
/// use tracing::{info, error};
///
/// #[tokio::main]
/// async fn main() {
///     let config = CallbackServerConfig::default();
///
///     match start_callback_server(config).await {
///         Ok(callback_stream) => {
///             info!("Callback server started successfully");
///             // Process callbacks from the stream
///         }
///         Err(e) => {
///             error!("Failed to start callback server: {}", e);
///             std::process::exit(1);
///         }
///     }
/// }
/// ```
///
/// ## Performance Considerations
///
/// - Uses tokio async runtime for high concurrency
/// - Callback processing is non-blocking via channels
/// - Memory usage scales with concurrent connections
///
/// ## Monitoring
///
/// The server provides several monitoring capabilities:
/// - Structured logging via tracing crate
/// - Request ID tracking for correlation
/// - Health check endpoint at `/health`
/// - Detailed error logging with context
///
/// ## Development Deployment
///
/// For development use:
/// - Server runs on localhost only for security
/// - Uses non-privileged port 8500
pub async fn start_callback_server(
    config: CallbackServerConfig,
) -> Result<impl Stream<Item = MomoUpdates>, Box<dyn Error>> {
    info!("Starting MTN MoMo Callback Server");
    info!("Host: {}, Port: {}", config.host, config.http_port);
    
    let (tx, mut rx) = mpsc::channel::<MomoUpdates>(100);
    
    // Create the application with routes and middleware
    let app = create_callback_routes()
        .with(poem::middleware::Tracing::default())
        .with(poem::middleware::Cors::new())
        .with(poem::middleware::Compression::default())
        .with(poem::middleware::RequestId::default())
        .with(AddData::new(tx.clone()));

    // Start the server on the HTTP port only
    let bind_address = format!("{}:{}", config.host, config.http_port);
    
    info!("Binding server to address: {}", bind_address);
    
    // Start HTTP server  
    tokio::spawn(async move {
        let listener = TcpListener::bind(&bind_address);
        
        match Server::new(listener)
            .run_with_graceful_shutdown(
                app,
                async {
                    tokio::signal::ctrl_c().await.ok();
                    info!("Received shutdown signal, stopping server...");
                },
                None,
            ).await {
            Ok(_) => info!("Server stopped successfully"),
            Err(e) => error!("Server error: {}", e),
        }
    });

    info!("MTN MoMo Callback Server started successfully");
    
    Ok(async_stream::stream! {
        while let Some(msg) = rx.recv().await {
            yield msg;
        }
    })
}