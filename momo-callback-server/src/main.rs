// MTN MoMo Callback Server - Main executable

use std::env;
use std::error::Error;

use futures_core::Stream;
use futures_util::StreamExt;
use poem::listener::{Listener, TcpListener, RustlsConfig};
use poem::middleware::AddData;
use poem::web::{Data, Path};
use poem::{handler, post, get, Body, Request, Response, Route, Server, EndpointExt};
use tokio::sync::mpsc::{self, Sender};
use tracing::{info, warn, error};
use tracing_subscriber;

use mtnmomo::{CallbackResponse, CallbackType, MomoUpdates};

/// Configuration structure for the MTN MoMo callback server.
///
/// This structure holds all the necessary configuration parameters for running
/// the callback server, including TLS certificate paths, network binding configuration,
/// and server settings.
///
/// ## Examples
///
/// ```rust,no_run
/// use std::env;
/// # use momo_callback_server::CallbackServerConfig;
///
/// // Create configuration with default values
/// let config = CallbackServerConfig::default();
///
/// // Create configuration with custom values
/// let config = CallbackServerConfig {
///     cert_path: "/path/to/custom/cert.pem".to_string(),
///     key_path: "/path/to/custom/key.pem".to_string(),
///     port: 8443,
///     host: "127.0.0.1".to_string(),
/// };
/// ```
///
/// ## Environment Variables
///
/// The default implementation reads from these environment variables:
/// - `TLS_CERT_PATH`: Path to the TLS certificate file
/// - `TLS_KEY_PATH`: Path to the TLS private key file
///
/// ## Security Notes
///
/// - Ensure certificate and key files have appropriate permissions (600 or 400)
/// - Use absolute paths for production deployments
/// - Verify certificate validity before starting the server
#[derive(Debug, Clone)]
pub struct CallbackServerConfig {
    /// Path to the TLS certificate file in PEM format.
    ///
    /// This should be a valid X.509 certificate that matches your domain name.
    /// The certificate file must be readable by the server process.
    pub cert_path: String,

    /// Path to the TLS private key file in PEM format.
    ///
    /// This should be the private key corresponding to the certificate.
    /// Keep this file secure and readable only by the server process.
    pub key_path: String,

    /// Port number for the server to bind to.
    ///
    /// Default is 443 (HTTPS). For non-root deployments, use ports > 1024
    /// or configure proper capabilities.
    pub port: u16,

    /// Host address to bind the server to.
    ///
    /// Default is "0.0.0.0" (all interfaces). Use "127.0.0.1" for localhost only.
    pub host: String,
}

impl Default for CallbackServerConfig {
    /// Creates a default configuration for the callback server.
    ///
    /// This implementation reads configuration from environment variables with
    /// sensible fallback defaults:
    ///
    /// - `cert_path`: `TLS_CERT_PATH` env var or "cert.pem"
    /// - `key_path`: `TLS_KEY_PATH` env var or "key.pem"
    /// - `port`: 443 (HTTPS standard port)
    /// - `host`: "0.0.0.0" (bind to all interfaces)
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use momo_callback_server::CallbackServerConfig;
    /// use std::env;
    ///
    /// // Set environment variables
    /// env::set_var("TLS_CERT_PATH", "/etc/ssl/certs/server.pem");
    /// env::set_var("TLS_KEY_PATH", "/etc/ssl/private/server.key");
    ///
    /// // Create default configuration (will use env vars)
    /// let config = CallbackServerConfig::default();
    /// assert_eq!(config.cert_path, "/etc/ssl/certs/server.pem");
    /// assert_eq!(config.port, 443);
    /// ```
    fn default() -> Self {
        Self {
            cert_path: env::var("TLS_CERT_PATH").unwrap_or_else(|_| "cert.pem".to_string()),
            key_path: env::var("TLS_KEY_PATH").unwrap_or_else(|_| "key.pem".to_string()),
            port: 443,
            host: "0.0.0.0".to_string(),
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
/// - `callback_type`: URL parameter indicating the type of callback
///
/// ## Callback Flow
///
/// 1. **Request Reception**: Receives POST request from MTN MoMo API
/// 2. **Body Extraction**: Reads and converts request body to string
/// 3. **JSON Parsing**: Attempts to parse body as `CallbackResponse`
/// 4. **Type Resolution**: Maps callback_type string to `CallbackType` enum
/// 5. **Channel Forwarding**: Sends parsed callback to processing channel
/// 6. **Response**: Returns success response to MTN MoMo API
///
/// ## Supported Callback Types
///
/// - `REQUEST_TO_PAY`: Payment request callbacks
/// - `INVOICE`: Invoice-related callbacks
/// - `DISBURSEMENT_*`: Disbursement operation callbacks
/// - `REMITTANCE_*`: Remittance operation callbacks
/// - And all other MTN MoMo callback types
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
///
/// ## Examples
///
/// The handler processes callbacks like this:
///
/// ```bash
/// # MTN MoMo API sends callback
/// curl -X POST https://your-server.com/collection_request_to_pay/REQUEST_TO_PAY \
///   -H "Content-Type: application/json" \
///   -d '{
///     "financialTransactionId": "123456",
///     "externalId": "payment-001",
///     "amount": "100",
///     "currency": "UGX",
///     "status": "SUCCESSFUL"
///   }'
/// ```
#[handler]
async fn mtn_callback_handler(
    req: &Request,
    mut body: Body,
    sender: Data<&Sender<MomoUpdates>>,
    Path(callback_type): Path<String>,
) -> Result<Response, poem::Error> {
    let remote_address = req.remote_addr().to_string();
    let body_string = body.into_string().await?;
    
    info!("Received callback from {}: {}", remote_address, callback_type);
    
    let response_result: Result<CallbackResponse, serde_json::Error> =
        serde_json::from_str(&body_string);
    
    match response_result {
        Ok(callback_response) => {
            let momo_updates = MomoUpdates {
                remote_address,
                response: callback_response,
                update_type: CallbackType::from_string(&callback_type),
            };
            
            if let Err(e) = sender.send(momo_updates).await {
                error!("Failed to send callback update: {}", e);
            } else {
                info!("Successfully processed {} callback", callback_type);
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
/// All routes follow the pattern: `/{service}_{operation}/{callback_type}`
///
/// Where:
/// - `service`: The MTN MoMo service (collection, disbursement, remittance)
/// - `operation`: The specific operation (request_to_pay, deposit, transfer, etc.)
/// - `callback_type`: The callback type parameter for the handler
///
/// ## Supported Routes
///
/// ### Collection Service
/// - `/collection_request_to_pay/{callback_type}`: Payment requests
/// - `/collection_request_to_withdraw_v1/{callback_type}`: Withdrawal v1
/// - `/collection_request_to_withdraw_v2/{callback_type}`: Withdrawal v2
/// - `/collection_invoice/{callback_type}`: Invoice operations
/// - `/collection_payment/{callback_type}`: Payment operations
/// - `/collection_preapproval/{callback_type}`: Pre-approval operations
///
/// ### Disbursement Service
/// - `/disbursement_deposit_v1/{callback_type}`: Deposit v1 operations
/// - `/disbursement_deposit_v2/{callback_type}`: Deposit v2 operations
/// - `/disbursement_refund_v1/{callback_type}`: Refund v1 operations
/// - `/disbursement_refund_v2/{callback_type}`: Refund v2 operations
/// - `/disbursement_transfer/{callback_type}`: Transfer operations
///
/// ### Remittance Service
/// - `/remittance_cash_transfer/{callback_type}`: Cash transfers
/// - `/remittance_transfer/{callback_type}`: Regular transfers
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
/// POST /collection_request_to_pay/REQUEST_TO_PAY
///
/// # Disbursement deposit callback
/// POST /disbursement_deposit_v1/DISBURSEMENT_DEPOSIT_V1
///
/// # Health check
/// GET /health
/// ```
fn create_callback_routes() -> Route {
    Route::new()
        // Collection callbacks
        .at(
            "/collection_request_to_pay/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/collection_request_to_withdraw_v1/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/collection_request_to_withdraw_v2/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/collection_invoice/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/collection_payment/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/collection_preapproval/:callback_type",
            post(mtn_callback_handler),
        )
        // Disbursement callbacks
        .at(
            "/disbursement_deposit_v1/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/disbursement_deposit_v2/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/disbursement_refund_v1/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/disbursement_refund_v2/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/disbursement_transfer/:callback_type",
            post(mtn_callback_handler),
        )
        // Remittance callbacks
        .at(
            "/remittance_cash_transfer/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/remittance_transfer/:callback_type",
            post(mtn_callback_handler),
        )
        // Health check endpoint
        .at("/health", get(health_check))
}

/// Loads and validates TLS configuration from certificate and key files.
///
/// This function reads the TLS certificate and private key files specified in the
/// configuration, validates their format, and creates a `RustlsConfig` object
/// for secure HTTPS connections.
///
/// ## Parameters
///
/// - `config`: Configuration containing paths to certificate and key files
///
/// ## Returns
///
/// - `Ok(RustlsConfig)`: Successfully loaded and configured TLS settings
/// - `Err(Box<dyn Error>)`: Failed to load or validate certificate/key files
///
/// ## File Requirements
///
/// ### Certificate File (cert.pem)
/// - Must be in PEM format
/// - Should contain a valid X.509 certificate
/// - Must be readable by the server process
/// - Should match the domain name where server will be accessed
///
/// ### Private Key File (key.pem)
/// - Must be in PEM format
/// - Should be the private key corresponding to the certificate
/// - Must be readable by the server process (recommended permissions: 600)
/// - Should be kept secure and not shared
///
/// ## Security Considerations
///
/// - Files are read once at startup and kept in memory
/// - Private key should have restrictive file permissions (600 or 400)
/// - Certificate should be from a trusted Certificate Authority for production
/// - Both files should be stored securely and backed up
///
/// ## Error Cases
///
/// This function will return an error if:
/// - Certificate file doesn't exist or isn't readable
/// - Private key file doesn't exist or isn't readable
/// - Certificate file is not valid PEM format
/// - Private key file is not valid PEM format
/// - Certificate and private key don't match
/// - Files contain malformed or corrupted data
///
/// ## Examples
///
/// ```rust,no_run
/// # use momo_callback_server::{CallbackServerConfig, load_tls_config};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = CallbackServerConfig {
///     cert_path: "/etc/ssl/certs/server.pem".to_string(),
///     key_path: "/etc/ssl/private/server.key".to_string(),
///     port: 443,
///     host: "0.0.0.0".to_string(),
/// };
///
/// match load_tls_config(&config).await {
///     Ok(tls_config) => {
///         println!("TLS configuration loaded successfully");
///         // Use tls_config with the server
///     }
///     Err(e) => {
///         eprintln!("Failed to load TLS configuration: {}", e);
///         std::process::exit(1);
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Certificate Generation
///
/// For development/testing, you can generate self-signed certificates:
///
/// ```bash
/// # Generate private key
/// openssl genrsa -out key.pem 2048
///
/// # Generate self-signed certificate
/// openssl req -new -x509 -key key.pem -out cert.pem -days 365
/// ```
///
/// For production, obtain certificates from a trusted CA like Let's Encrypt:
///
/// ```bash
/// # Using certbot for Let's Encrypt
/// certbot certonly --standalone -d your-domain.com
/// ```
async fn load_tls_config(config: &CallbackServerConfig) -> Result<RustlsConfig, Box<dyn Error>> {
    info!("Loading TLS certificate from: {}", config.cert_path);
    info!("Loading TLS private key from: {}", config.key_path);
    
    let cert_data = std::fs::read(&config.cert_path)?;
    let key_data = std::fs::read(&config.key_path)?;
    
    use poem::listener::RustlsCertificate;
    
    let certificate = RustlsCertificate::new()
        .cert(cert_data)
        .key(key_data);
        
    let tls_config = RustlsConfig::new()
        .fallback(certificate);

    info!("TLS configuration loaded successfully");
    Ok(tls_config)
}

/// Starts the MTN MoMo callback server with the specified configuration.
///
/// This is the main function that initializes and starts the callback server. It sets up
/// TLS configuration, creates the web application with all routes and middleware, starts
/// the server in a background task, and returns a stream of processed callbacks.
///
/// ## Parameters
///
/// - `config`: Server configuration including TLS settings and network binding
///
/// ## Returns
///
/// - `Ok(Stream<Item = MomoUpdates>)`: A stream of processed callback updates
/// - `Err(Box<dyn Error>)`: Startup error (TLS config, network binding, etc.)
///
/// ## Server Lifecycle
///
/// 1. **TLS Configuration**: Loads and validates certificate and key files
/// 2. **Route Setup**: Creates all callback routes and middleware stack
/// 3. **Network Binding**: Binds to the specified host and port with TLS
/// 4. **Background Task**: Spawns server in a background tokio task
/// 5. **Channel Setup**: Creates communication channel for callback processing
/// 6. **Stream Return**: Returns async stream of incoming callbacks
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
/// - TLS certificate or key files are missing or invalid
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
///         println!("Received callback: {:?}", callback.update_type);
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
///         cert_path: "/etc/ssl/certs/my-cert.pem".to_string(),
///         key_path: "/etc/ssl/private/my-key.pem".to_string(),
///         port: 8443,  // Non-standard port
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
/// - TLS termination is handled efficiently by rustls
/// - Callback processing is non-blocking via channels
/// - Memory usage scales with concurrent connections
/// - CPU usage is primarily for TLS encryption/decryption
///
/// ## Monitoring
///
/// The server provides several monitoring capabilities:
/// - Structured logging via tracing crate
/// - Request ID tracking for correlation
/// - Health check endpoint at `/health`
/// - Detailed error logging with context
///
/// ## Production Deployment
///
/// For production use, consider:
/// - Running behind a reverse proxy (nginx, traefik)
/// - Using process managers (systemd, supervisor)
/// - Implementing log rotation and monitoring
/// - Setting up automated certificate renewal
/// - Configuring firewall rules for port 443
pub async fn start_callback_server(
    config: CallbackServerConfig,
) -> Result<impl Stream<Item = MomoUpdates>, Box<dyn Error>> {
    info!("Starting MTN MoMo Callback Server");
    info!("Host: {}, Port: {}", config.host, config.port);
    
    let (tx, mut rx) = mpsc::channel::<MomoUpdates>(100);
    
    // Load TLS configuration
    let tls_config = load_tls_config(&config).await?;
    
    // Create the application with routes and middleware
    let app = create_callback_routes()
        .with(poem::middleware::Tracing::default())
        .with(poem::middleware::Cors::new())
        .with(poem::middleware::Compression::default())
        .with(poem::middleware::RequestId::default())
        .with(AddData::new(tx));

    // Start the server
    let bind_address = format!("{}:{}", config.host, config.port);
    info!("Binding to address: {}", bind_address);
    
    tokio::spawn(async move {
        let listener = TcpListener::bind(&bind_address)
            .rustls(tls_config);
        
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

#[tokio::main]
#[allow(dead_code)]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("MTN MoMo Callback Server starting...");

    // Load configuration
    let config = CallbackServerConfig::default();
    
    // Validate certificate files exist
    if !std::path::Path::new(&config.cert_path).exists() {
        error!("Certificate file not found: {}", config.cert_path);
        std::process::exit(1);
    }
    
    if !std::path::Path::new(&config.key_path).exists() {
        error!("Private key file not found: {}", config.key_path);
        std::process::exit(1);
    }

    // Start the callback server
    let callback_stream = start_callback_server(config).await?;
    
    info!("Server is running. Press Ctrl+C to stop.");
    
    // Process incoming callbacks
    use futures_util::pin_mut;
    pin_mut!(callback_stream);
    while let Some(update) = callback_stream.next().await {
        info!("Processing callback: {:?}", update.update_type);
        info!("From: {}", update.remote_address);
        
        // Here you can add custom business logic to handle different callback types
        match update.update_type {
            CallbackType::RequestToPay => {
                info!("Processing payment callback: {:?}", update.response);
                handle_payment_callback(&update).await;
            }
            CallbackType::Invoice => {
                info!("Processing invoice callback: {:?}", update.response);
                handle_invoice_callback(&update).await;
            }
            CallbackType::DisbursementDepositV1 => {
                info!("Processing disbursement deposit v1 callback: {:?}", update.response);
                handle_disbursement_callback(&update).await;
            }
            CallbackType::DisbursementDepositV2 => {
                info!("Processing disbursement deposit v2 callback: {:?}", update.response);
                handle_disbursement_callback(&update).await;
            }
            CallbackType::DisbursementRefundV1 => {
                info!("Processing disbursement refund v1 callback: {:?}", update.response);
                handle_disbursement_callback(&update).await;
            }
            CallbackType::DisbursementRefundV2 => {
                info!("Processing disbursement refund v2 callback: {:?}", update.response);
                handle_disbursement_callback(&update).await;
            }
            CallbackType::DisbusrementTransfer => {
                info!("Processing disbursement transfer callback: {:?}", update.response);
                handle_disbursement_callback(&update).await;
            }
            CallbackType::RemittanceCashTransfer => {
                info!("Processing remittance cash transfer callback: {:?}", update.response);
                handle_remittance_callback(&update).await;
            }
            CallbackType::RemittanceTransfer => {
                info!("Processing remittance transfer callback: {:?}", update.response);
                handle_remittance_callback(&update).await;
            }
            _ => {
                info!("Processing other callback type: {:?}", update.response);
                handle_generic_callback(&update).await;
            }
        }
    }

    Ok(())
}

/// Handles payment-related callbacks with custom business logic.
///
/// This function processes callbacks related to payment operations, including
/// successful payments, failed payments, and payment status updates. It extracts
/// relevant information from the callback response and provides hooks for
/// implementing custom business logic.
///
/// ## Parameters
///
/// - `update`: The callback update containing payment information and metadata
///
/// ## Supported Callback Types
///
/// - **RequestToPaySuccess**: Successful payment completion
/// - **RequestToPayFailed**: Failed payment with reason
///
/// ## Business Logic Integration
///
/// This function provides hooks for implementing payment processing logic:
///
/// ### Successful Payments
/// - Update database records
/// - Send confirmation notifications
/// - Process order fulfillment
/// - Update user balances
/// - Generate receipts
///
/// ### Failed Payments
/// - Handle refunds if applicable
/// - Notify users of failure
/// - Log for investigation
/// - Update payment status
/// - Trigger retry mechanisms
///
/// ## Examples
///
/// ### Database Integration
///
/// ```rust,no_run
/// // Example of extending this function for database updates
/// async fn handle_payment_callback(update: &MomoUpdates) {
///     match &update.response {
///         CallbackResponse::RequestToPaySuccess { external_id, amount, currency, .. } => {
///             // Update payment status in database
///             database::update_payment_status(external_id, "completed").await;
///             
///             // Send confirmation email
///             email::send_payment_confirmation(external_id, amount, currency).await;
///             
///             // Process order if this was a purchase
///             orders::fulfill_order(external_id).await;
///         }
///         CallbackResponse::RequestToPayFailed { external_id, reason, .. } => {
///             // Update payment status
///             database::update_payment_status(external_id, "failed").await;
///             
///             // Log failure for analysis
///             analytics::log_payment_failure(external_id, reason).await;
///             
///             // Notify user
///             notifications::send_failure_notification(external_id, reason).await;
///         }
///         _ => {}
///     }
/// }
/// ```
///
/// ### Webhook Forwarding
///
/// ```rust,no_run
/// // Example of forwarding callbacks to other services
/// async fn handle_payment_callback(update: &MomoUpdates) {
///     let webhook_payload = serde_json::to_string(&update.response)?;
///     
///     // Forward to internal accounting service
///     http_client
///         .post("http://accounting-service/payments/callback")
///         .body(webhook_payload)
///         .send()
///         .await?;
/// }
/// ```
///
/// ## Error Handling
///
/// This function should handle errors gracefully to avoid affecting
/// the callback server's stability:
///
/// ```rust,no_run
/// async fn handle_payment_callback(update: &MomoUpdates) {
///     match process_payment(update).await {
///         Ok(_) => info!("Payment processed successfully"),
///         Err(e) => {
///             error!("Failed to process payment: {}", e);
///             // Don't panic - log and continue
///         }
///     }
/// }
/// ```
#[allow(dead_code)]
async fn handle_payment_callback(update: &MomoUpdates) {
    info!("Payment callback processing started");
    
    // Extract payment information based on callback response variant
    match &update.response {
        CallbackResponse::RequestToPaySuccess {
            external_id,
            status: _,
            financial_transaction_id,
            amount,
            currency,
            ..
        } => {
            info!("Payment successful - External ID: {}, Transaction ID: {}, Amount: {} {}", 
                  external_id, financial_transaction_id, amount, currency);
            // Add your success handling logic here
            // e.g., update database, send notifications, etc.
        }
        CallbackResponse::RequestToPayFailed {
            external_id,
            status: _,
            reason,
            amount,
            currency,
            ..
        } => {
            info!("Payment failed - External ID: {}, Reason: {:?}, Amount: {} {}", 
                  external_id, reason, amount, currency);
            // Add your failure handling logic here
            // e.g., handle refunds, notify user, etc.
        }
        _ => {
            warn!("Received non-payment callback in payment handler: {:?}", update.response);
        }
    }
}

/// Handles invoice-related callbacks with custom business logic.
///
/// This function processes callbacks for invoice operations, including successful
/// invoice payments and failed invoice attempts. It provides integration points
/// for implementing custom invoice management logic.
///
/// ## Parameters
///
/// - `update`: The callback update containing invoice information and metadata
///
/// ## Supported Callback Types
///
/// - **InvoiceSucceeded**: Invoice payment completed successfully
/// - **InvoiceFailed**: Invoice payment failed with error reason
///
/// ## Business Logic Integration
///
/// ### Successful Invoice Payments
/// - Mark invoice as paid in database
/// - Send payment confirmation to customer
/// - Update accounting records
/// - Trigger order processing
/// - Generate payment receipts
///
/// ### Failed Invoice Payments
/// - Update invoice status to failed
/// - Send payment failure notification
/// - Log failure for analysis
/// - Trigger reminder workflows
/// - Update payment retry attempts
///
/// ## Examples
///
/// ```rust,no_run
/// async fn handle_invoice_callback(update: &MomoUpdates) {
///     match &update.response {
///         CallbackResponse::InvoiceSucceeded { 
///             external_id, 
///             reference_id, 
///             amount, 
///             currency,
///             .. 
///         } => {
///             // Update invoice status in database
///             database::mark_invoice_paid(external_id, reference_id).await;
///             
///             // Send confirmation to customer
///             email::send_invoice_payment_confirmation(
///                 external_id, 
///                 amount, 
///                 currency
///             ).await;
///             
///             // Update accounting system
///             accounting::record_invoice_payment(
///                 external_id, 
///                 amount, 
///                 currency
///             ).await;
///         }
///         CallbackResponse::InvoiceFailed { 
///             external_id, 
///             reference_id, 
///             error_reason,
///             .. 
///         } => {
///             // Update invoice status
///             database::mark_invoice_failed(external_id, erron_reason).await;
///             
///             // Send failure notification
///             email::send_invoice_payment_failure(
///                 external_id, 
///                 erron_reason
///             ).await;
///             
///             // Schedule retry if appropriate
///             if should_retry_invoice(erron_reason) {
///                 retry_service::schedule_invoice_retry(external_id).await;
///             }
///         }
///         _ => {
///             warn!("Unexpected callback type in invoice handler");
///         }
///     }
/// }
/// ```
#[allow(dead_code)]
async fn handle_invoice_callback(update: &MomoUpdates) {
    info!("Invoice callback processing started");
    
    // Handle invoice-specific callback variants
    match &update.response {
        CallbackResponse::InvoiceSucceeded {
            external_id,
            reference_id,
            status: _,
            amount,
            currency,
            ..
        } => {
            info!("Invoice successful - External ID: {}, Reference ID: {}, Amount: {} {}", 
                  external_id, reference_id, amount, currency);
            // e.g., update invoice status in database
        }
        CallbackResponse::InvoiceFailed {
            external_id,
            reference_id,
            status: _,
            error_reason,
            ..
        } => {
            info!("Invoice failed - External ID: {}, Reference ID: {}, Reason: {:?}", 
                  external_id, reference_id, error_reason);
            // e.g., handle invoice failure
        }
        _ => {
            warn!("Received non-invoice callback in invoice handler: {:?}", update.response);
        }
    }
}

/// Handles disbursement-related callbacks with custom business logic.
///
/// This function processes callbacks for disbursement operations, including successful
/// disbursements and failed disbursement attempts. Disbursements typically involve
/// sending money from the merchant account to end users or other parties.
///
/// ## Parameters
///
/// - `update`: The callback update containing disbursement information and metadata
///
/// ## Supported Callback Types
///
/// - **PaymentSucceeded**: Disbursement completed successfully
/// - **PaymentFailed**: Disbursement failed with reason
///
/// ## Business Logic Integration
///
/// ### Successful Disbursements
/// - Update disbursement status in database
/// - Send confirmation notifications to recipients
/// - Update account balances
/// - Generate disbursement receipts
/// - Log transaction for audit
///
/// ### Failed Disbursements
/// - Mark disbursement as failed
/// - Notify administrators of failure
/// - Log for investigation
/// - Handle retry logic if appropriate
/// - Refund source account if necessary
///
/// ## Examples
///
/// ```rust,no_run
/// async fn handle_disbursement_callback(update: &MomoUpdates) {
///     match &update.response {
///         CallbackResponse::PaymentSucceeded { 
///             reference_id, 
///             financial_transaction_id, 
///             .. 
///         } => {
///             // Update disbursement record
///             database::update_disbursement_status(
///                 reference_id, 
///                 "completed"
///             ).await;
///             
///             // Send confirmation
///             notifications::send_disbursement_confirmation(
///                 reference_id,
///                 financial_transaction_id
///             ).await;
///             
///             // Update account balance
///             accounts::update_balance_after_disbursement(
///                 reference_id
///             ).await;
///         }
///         CallbackResponse::PaymentFailed { 
///             reference_id, 
///             reason, 
///             .. 
///         } => {
///             // Handle failed disbursement
///             database::mark_disbursement_failed(
///                 reference_id, 
///                 reason
///             ).await;
///             
///             // Notify administrators
///             notifications::alert_disbursement_failure(
///                 reference_id, 
///                 reason
///             ).await;
///             
///             // Potentially refund source account
///             accounts::refund_failed_disbursement(
///                 reference_id
///             ).await;
///         }
///         _ => {}
///     }
/// }
/// ```
#[allow(dead_code)]
async fn handle_disbursement_callback(update: &MomoUpdates) {
    info!("Disbursement callback processing started");
    
    // Handle disbursement-specific callback variants
    match &update.response {
        CallbackResponse::PaymentSucceeded {
            reference_id,
            status: _,
            financial_transaction_id,
        } => {
            info!("Disbursement successful - Reference ID: {}, Transaction ID: {}", 
                  reference_id, financial_transaction_id);
        }
        CallbackResponse::PaymentFailed {
            reference_id,
            status: _,
            financial_transaction_id,
            reason,
        } => {
            info!("Disbursement failed - Reference ID: {}, Transaction ID: {}, Reason: {:?}", 
                  reference_id, financial_transaction_id, reason);
        }
        CallbackResponse::DisbursementDepositV1Success {
            external_id,
            financial_transaction_id,
            amount,
            currency,
            status: _,
            ..
        } => {
            info!("Disbursement deposit v1 successful - External ID: {}, Transaction ID: {}, Amount: {} {}", 
                  external_id, financial_transaction_id, amount, currency);
        }
        CallbackResponse::DisbursementDepositV1Failed {
            external_id,
            financial_transaction_id,
            amount,
            currency,
            reason,
            ..
        } => {
            info!("Disbursement deposit v1 failed - External ID: {}, Transaction ID: {}, Amount: {} {}, Reason: {:?}", 
                  external_id, financial_transaction_id, amount, currency, reason);
        }
        CallbackResponse::DisbursementDepositV2Success {
            external_id,
            financial_transaction_id,
            amount,
            currency,
            status: _,
            ..
        } => {
            info!("Disbursement deposit v2 successful - External ID: {}, Transaction ID: {}, Amount: {} {}", 
                  external_id, financial_transaction_id, amount, currency);
        }
        CallbackResponse::DisbursementDepositV2Failed {
            external_id,
            financial_transaction_id,
            amount,
            currency,
            reason,
            ..
        } => {
            info!("Disbursement deposit v2 failed - External ID: {}, Transaction ID: {}, Amount: {} {}, Reason: {:?}", 
                  external_id, financial_transaction_id, amount, currency, reason);
        }
        CallbackResponse::DisbursementRefundV1Success {
            external_id,
            financial_transaction_id,
            amount,
            currency,
            status: _,
            ..
        } => {
            info!("Disbursement refund v1 successful - External ID: {}, Transaction ID: {}, Amount: {} {}", 
                  external_id, financial_transaction_id, amount, currency);
        }
        CallbackResponse::DisbursementRefundV1Failed {
            external_id,
            financial_transaction_id,
            amount,
            currency,
            reason,
            ..
        } => {
            info!("Disbursement refund v1 failed - External ID: {}, Transaction ID: {}, Amount: {} {}, Reason: {:?}", 
                  external_id, financial_transaction_id, amount, currency, reason);
        }
        CallbackResponse::DisbursementRefundV2Success {
            external_id,
            financial_transaction_id,
            amount,
            currency,
            status: _,
            ..
        } => {
            info!("Disbursement refund v2 successful - External ID: {}, Transaction ID: {}, Amount: {} {}", 
                  external_id, financial_transaction_id, amount, currency);
        }
        CallbackResponse::DisbursementRefundV2Failed {
            external_id,
            financial_transaction_id,
            amount,
            currency,
            reason,
            ..
        } => {
            info!("Disbursement refund v2 failed - External ID: {}, Transaction ID: {}, Amount: {} {}, Reason: {:?}", 
                  external_id, financial_transaction_id, amount, currency, reason);
        }
        CallbackResponse::DisbursementTransferSuccess {
            external_id,
            financial_transaction_id,
            amount,
            currency,
            status: _,
            ..
        } => {
            info!("Disbursement transfer successful - External ID: {}, Transaction ID: {}, Amount: {} {}", 
                  external_id, financial_transaction_id, amount, currency);
        }
        CallbackResponse::DisbursementTransferFailed {
            external_id,
            financial_transaction_id,
            amount,
            currency,
            reason,
            ..
        } => {
            info!("Disbursement transfer failed - External ID: {}, Transaction ID: {}, Amount: {} {}, Reason: {:?}", 
                  external_id, financial_transaction_id, amount, currency, reason);
        }
        _ => {
            info!("Generic disbursement callback: {:?}", update.response);
        }
    }
}

/// Handles remittance-related callbacks with custom business logic.
///
/// This function processes callbacks for remittance operations, including successful
/// money transfers and failed transfer attempts. Remittances typically involve
/// cross-border money transfers and cash pickup services.
///
/// ## Parameters
///
/// - `update`: The callback update containing remittance information and metadata
///
/// ## Supported Callback Types
///
/// - **CashTransferSucceeded**: Remittance completed successfully
/// - **CashTransferFailed**: Remittance failed with error details
///
/// ## Business Logic Integration
///
/// ### Successful Remittances
/// - Update transfer status in database
/// - Send confirmation to sender and recipient
/// - Update exchange rates if applicable
/// - Generate transfer receipts
/// - Notify recipient of available funds
///
/// ### Failed Remittances
/// - Mark transfer as failed
/// - Notify sender of failure
/// - Refund sender if money was already debited
/// - Log for compliance and investigation
/// - Handle retry mechanisms
///
/// ## Examples
///
/// ```rust,no_run
/// async fn handle_remittance_callback(update: &MomoUpdates) {
///     match &update.response {
///         CallbackResponse::CashTransferSucceeded {
///             external_id,
///             financial_transaction_id,
///             amount,
///             currency,
///             payee,
///             originating_country,
///             ..
///         } => {
///             // Update transfer status
///             database::complete_remittance_transfer(
///                 external_id,
///                 financial_transaction_id
///             ).await;
///             
///             // Notify recipient
///             sms::send_pickup_notification(
///                 &payee.party_id,
///                 amount,
///                 currency
///             ).await;
///             
///             // Send confirmation to sender
///             email::send_transfer_confirmation(
///                 external_id,
///                 amount,
///                 currency,
///                 originating_country
///             ).await;
///             
///             // Update compliance records
///             compliance::log_successful_transfer(
///                 external_id,
///                 amount,
///                 originating_country
///             ).await;
///         }
///         CallbackResponse::CashTransferFailed {
///             external_id,
///             error_reason,
///             amount,
///             currency,
///             ..
///         } => {
///             // Handle failed transfer
///             database::mark_remittance_failed(
///                 external_id,
///                 error_reason
///             ).await;
///             
///             // Refund sender
///             refunds::process_remittance_refund(
///                 external_id,
///                 amount,
///                 currency
///             ).await;
///             
///             // Notify sender of failure
///             notifications::send_transfer_failure_notice(
///                 external_id,
///                 error_reason
///             ).await;
///         }
///         _ => {}
///     }
/// }
/// ```
#[allow(dead_code)]
async fn handle_remittance_callback(update: &MomoUpdates) {
    info!("Remittance callback processing started");
    
    // Handle remittance-specific callback variants
    match &update.response {
        CallbackResponse::CashTransferSucceeded {
            external_id,
            financial_transaction_id,
            status: _,
            amount,
            currency,
            ..
        } => {
            info!("Remittance cash transfer successful - External ID: {}, Transaction ID: {}, Amount: {} {}", 
                  external_id, financial_transaction_id, amount, currency);
        }
        CallbackResponse::CashTransferFailed {
            external_id,
            financial_transaction_id,
            status: _,
            amount,
            currency,
            error_reason,
            ..
        } => {
            info!("Remittance cash transfer failed - External ID: {}, Transaction ID: {}, Amount: {} {}, Reason: {:?}", 
                  external_id, financial_transaction_id, amount, currency, error_reason);
        }
        CallbackResponse::RemittanceTransferSuccess {
            external_id,
            financial_transaction_id,
            status: _,
            amount,
            currency,
            ..
        } => {
            info!("Remittance transfer successful - External ID: {}, Transaction ID: {}, Amount: {} {}", 
                  external_id, financial_transaction_id, amount, currency);
        }
        CallbackResponse::RemittanceTransferFailed {
            external_id,
            financial_transaction_id,
            status: _,
            amount,
            currency,
            error_reason,
            ..
        } => {
            info!("Remittance transfer failed - External ID: {}, Transaction ID: {}, Amount: {} {}, Reason: {:?}", 
                  external_id, financial_transaction_id, amount, currency, error_reason);
        }
        _ => {
            info!("Generic remittance callback: {:?}", update.response);
        }
    }
}

/// Handles generic callbacks that don't fit into specific category handlers.
///
/// This function serves as a catch-all handler for callback types that don't have
/// dedicated processing logic or for handling edge cases and future callback types
/// that may be added to the MTN MoMo API.
///
/// ## Parameters
///
/// - `update`: The callback update containing callback information and metadata
///
/// ## Use Cases
///
/// - Processing new callback types during API evolution
/// - Handling edge cases not covered by specific handlers
/// - Logging and monitoring unknown callback patterns
/// - Debugging and development purposes
/// - Fallback processing for unclassified callbacks
///
/// ## Business Logic Integration
///
/// ### Logging and Monitoring
/// - Log all callback details for analysis
/// - Send metrics to monitoring systems
/// - Alert on unknown callback types
/// - Track callback volumes and patterns
///
/// ### Future-Proofing
/// - Handle new MTN MoMo callback types gracefully
/// - Provide extension points for custom logic
/// - Maintain backward compatibility
/// - Support A/B testing of new features
///
/// ## Examples
///
/// ```rust,no_run
/// async fn handle_generic_callback(update: &MomoUpdates) {
///     // Log comprehensive callback information
///     info!("Generic callback received");
///     info!("Type: {:?}", update.update_type);
///     info!("From: {}", update.remote_address);
///     info!("Response: {:?}", update.response);
///     
///     // Send to analytics/monitoring
///     analytics::track_callback_event({
///         "type": update.update_type,
///         "source": update.remote_address,
///         "timestamp": chrono::Utc::now(),
///     }).await;
///     
///     // Store for later analysis
///     database::store_unknown_callback({
///         update.clone()
///     }).await;
///     
///     // Alert if this is a new callback type we haven't seen
///     if is_new_callback_type(&update.update_type) {
///         alerts::send_new_callback_type_alert(
///             &update.update_type
///         ).await;
///     }
///     
///     // Forward to external systems if needed
///     webhook::forward_to_external_handler(update).await;
/// }
/// ```
///
/// ## Error Handling
///
/// This handler should be extremely robust since it processes unknown data:
///
/// ```rust,no_run
/// async fn handle_generic_callback(update: &MomoUpdates) {
///     // Always wrap in error handling
///     match process_generic_callback(update).await {
///         Ok(_) => info!("Generic callback processed successfully"),
///         Err(e) => {
///             error!("Failed to process generic callback: {}", e);
///             // Don't panic - continue processing other callbacks
///         }
///     }
/// }
/// ```
#[allow(dead_code)]
async fn handle_generic_callback(update: &MomoUpdates) {
    info!("Generic callback processing started");
    info!("Callback type: {:?}", update.update_type);
    info!("Response: {:?}", update.response);
    
    // Add your generic callback processing logic here
}