use crate::Options;
use trust_dns_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};

/// DNS Request Handler
#[derive(Clone, Debug)]
pub struct Handler {
    /// Request counter, incremented on every successful request.
    pub counter: Arc<AtomicU64>,
    /// Domain to serve DNS responses for (requests for other domains are silently ignored).
    pub root_zone: LowerName,
    /// Zone name for counter (counter.dnsfun.dev)
    pub counter_zone: LowerName,
    /// Zone name for myip (myip.dnsfun.dev)
    pub myip_zone: LowerName,
    /// Zone name for hello (hello.dnsfun.dev)
    pub hello_zone: LowerName,
}

impl Handler {
    /// Handle request, returning ResponseInfo if response was successfully sent, or an error.
    async fn do_handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        response: R,
    ) -> Result<ResponseInfo, Error> {
        // make sure the request is a query
        if request.op_code() != OpCode::Query {
            return Err(Error::InvalidOpCode(request.op_code()));
        }

        // make sure the message type is a query
        if request.message_type() != MessageType::Query {
            return Err(Error::InvalidMessageType(request.message_type()));
        }

        match request.query().name() {
            name if self.myip_zone.zone_of(name) => {
                self.do_handle_request_myip(request, response).await
            }
            name if self.counter_zone.zone_of(name) => {
                self.do_handle_request_counter(request, response).await
            }
            name if self.hello_zone.zone_of(name) => {
                self.do_handle_request_hello(request, response).await
            }
            name if self.root_zone.zone_of(name) => {
                self.do_handle_request_default(request, response).await
            }
            name => Err(Error::InvalidZone(name.clone())),
        }
    }
}

#[async_trait::async_trait]
impl RequestHandler for Handler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        response: R,
    ) -> ResponseInfo {
        // try to handle request
        match self.do_handle_request(request, response).await {
            Ok(info) => info,
            Err(error) => {
                error!("Error in RequestHandler: {error}");
                let mut header = Header::new();
                header.set_response_code(ResponseCode::ServFail);
                header.into()
            }
        }
    }
}
