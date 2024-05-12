pub mod proxy {

    use pingora::{connectors::http::Connector, prelude::*};
    use pingora_http::RequestHeader;
    use regex::Regex;

    pub async fn client() -> Result<String> {
        /*  let server_conf = ServerConf::load_from_yaml("pingora_conf.yaml").unwrap();
               let connector = Connector::new(Some(ConnectorOptions::from_server_conf(&server_conf)));
        */

        let connector = Connector::new(None);
        // create the HTTP session
        let peer_addr = "23.214.139.102:443";
        //let peer_addr = "1.1.1.1:443";
        let mut peer = HttpPeer::new(peer_addr, true, "ng.soccerway.com".into());

        peer.options.set_http_version(2, 1);
        let (mut http, reused) = connector.get_http_session(&peer).await?;
        assert!(!reused);

        // perform a GET request
        let mut new_request = RequestHeader::build("GET", b"/", None)?;
        new_request.insert_header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) ",
        )?;

        new_request.insert_header("Host", "ng.soccerway.com")?;
        http.write_request_header(Box::new(new_request)).await?;

        // Servers usually don't respond until the full request body is read.
        http.finish_request_body().await?;
        http.read_response_header().await?;

        // display the headers from the response
        if let Some(header) = http.response_header() {
            println!("header is {header:#?}");
        } else {
            return Error::e_explain(ErrorType::InvalidHTTPHeader, "No response header");
        };

        // collect the response body
        let mut response_body = String::new();
        while let Some(chunk) = http.read_response_body().await? {
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

        Ok(response_body)
    }
}
