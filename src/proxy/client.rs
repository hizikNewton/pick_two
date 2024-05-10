pub mod proxy {

    use pingora::{
        connectors::{http::Connector, ConnectorOptions},
        prelude::*,
        server::configuration::ServerConf,
        upstreams::peer::Peer,
    };
    use pingora_http::RequestHeader;
    use regex::Regex;

    pub async fn client() -> Result<()> {
        let server_conf = ServerConf::load_from_yaml("pingora_conf.yaml").unwrap();
        let connector = Connector::new(Some(ConnectorOptions::from_server_conf(&server_conf)));

        // create the HTTP session
        let peer_addr = "1.1.1.1:443";
        //let peer_addr = "1.1.1.1:443";
        let mut peer = HttpPeer::new(peer_addr, true, "one.one.one.one".into());

        log::info!("we are crawling ,{:?}", peer.address());
        peer.options.set_http_version(2, 1);
        let (mut http, _reused) = connector.get_http_session(&peer).await?;

        // perform a GET request
        let mut new_request = RequestHeader::build("GET", b"/", None)?;
        new_request.insert_header("Host", "one.one.one.one")?;
        http.write_request_header(Box::new(new_request)).await?;

        // Servers usually don't respond until the full request body is read.
        http.finish_request_body().await?;
        http.read_response_header().await?;

        // display the headers from the response
        if let Some(header) = http.response_header() {
            println!("{header:#?}");
        } else {
            return Error::e_explain(ErrorType::InvalidHTTPHeader, "No response header");
        };

        // collect the response body
        let mut response_body = String::new();
        while let Some(chunk) = http.read_response_body().await? {
            println!("smsm {response_body}");
            response_body.push_str(&String::from_utf8_lossy(&chunk));
        }

        // verify that the response body is valid HTML by displaying the page <title>
        let re = Regex::new(r"<title>(.*?)</title>")
            .or_err(ErrorType::InternalError, "Failed to compile regex")?;
        if let Some(title) = re
            .captures(&response_body)
            .and_then(|caps| caps.get(1).map(|match_| match_.as_str()))
        {
            println!("Page Title: {title}");
        } else {
            return Error::e_explain(
                ErrorType::new("InvalidHTML"),
                "No <title> found in response body",
            );
        }

        // gracefully release the connection
        connector
            .release_http_session(http, &peer, Some(std::time::Duration::from_secs(5)))
            .await;

        Ok(())
    }
}
