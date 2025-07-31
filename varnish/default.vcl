# Varnish configuration for CDN caching
vcl 4.1;

import std;
import directors;

# Backend definitions
backend frontend {
    .host = "raffle-frontend-service";
    .port = "80";
    .connect_timeout = 5s;
    .first_byte_timeout = 30s;
    .between_bytes_timeout = 5s;
    .max_connections = 50;
    .probe = {
        .url = "/health";
        .timeout = 5s;
        .interval = 10s;
        .window = 5;
        .threshold = 3;
    };
}

backend api {
    .host = "raffle-backend-service";
    .port = "80";
    .connect_timeout = 5s;
    .first_byte_timeout = 60s;
    .between_bytes_timeout = 10s;
    .max_connections = 100;
    .probe = {
        .url = "/health";
        .timeout = 5s;
        .interval = 10s;
        .window = 5;
        .threshold = 3;
    };
}

# Access Control List for purging
acl purge {
    "localhost";
    "127.0.0.1";
    "10.0.0.0"/8;
    "172.16.0.0"/12;
    "192.168.0.0"/16;
}

# Receive routine
sub vcl_recv {
    # Set client IP
    if (req.http.X-Forwarded-For) {
        set req.http.X-Forwarded-For = req.http.X-Forwarded-For + ", " + client.ip;
    } else {
        set req.http.X-Forwarded-For = client.ip;
    }

    # Handle PURGE requests
    if (req.method == "PURGE") {
        if (!client.ip ~ purge) {
            return (synth(405, "Method not allowed"));
        }
        return (purge);
    }

    # Only handle GET, HEAD, POST, PUT, DELETE, and OPTIONS
    if (req.method != "GET" &&
        req.method != "HEAD" &&
        req.method != "POST" &&
        req.method != "PUT" &&
        req.method != "DELETE" &&
        req.method != "OPTIONS") {
        return (synth(405, "Method not allowed"));
    }

    # Route to appropriate backend
    if (req.url ~ "^/api/") {
        set req.backend_hint = api;
        # Don't cache API requests by default
        if (req.method != "GET" && req.method != "HEAD") {
            return (pass);
        }
        # Cache only specific API endpoints
        if (req.url ~ "^/api/v1/(raffles|items|categories)$" && req.method == "GET") {
            # Cache public API endpoints
            unset req.http.Cookie;
        } else {
            return (pass);
        }
    } else {
        set req.backend_hint = frontend;
    }

    # Handle CORS preflight requests
    if (req.method == "OPTIONS") {
        return (synth(200, "OK"));
    }

    # Remove cookies for static assets
    if (req.url ~ "\.(css|js|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot|webp|avif)$") {
        unset req.http.Cookie;
        unset req.http.Authorization;
    }

    # Remove tracking parameters
    if (req.url ~ "(\?|&)(utm_source|utm_medium|utm_campaign|utm_content|utm_term|gclid|fbclid)=") {
        set req.url = regsuball(req.url, "(\?|&)(utm_source|utm_medium|utm_campaign|utm_content|utm_term|gclid|fbclid)=[^&]*", "");
        set req.url = regsuball(req.url, "(\?|&)$", "");
    }

    # Normalize Accept-Encoding header
    if (req.http.Accept-Encoding) {
        if (req.url ~ "\.(jpg|jpeg|png|gif|gz|tgz|bz2|tbz|mp3|ogg|swf|flv)$") {
            # No compression for already compressed files
            unset req.http.Accept-Encoding;
        } elsif (req.http.Accept-Encoding ~ "gzip") {
            set req.http.Accept-Encoding = "gzip";
        } elsif (req.http.Accept-Encoding ~ "deflate") {
            set req.http.Accept-Encoding = "deflate";
        } else {
            unset req.http.Accept-Encoding;
        }
    }

    return (hash);
}

# Backend response routine
sub vcl_backend_response {
    # Set cache headers based on content type
    if (bereq.url ~ "\.(css|js)$") {
        # Cache CSS and JS for 1 year
        set beresp.ttl = 365d;
        set beresp.http.Cache-Control = "public, max-age=31536000, immutable";
    } elsif (bereq.url ~ "\.(png|jpg|jpeg|gif|ico|svg|webp|avif)$") {
        # Cache images for 1 month
        set beresp.ttl = 30d;
        set beresp.http.Cache-Control = "public, max-age=2592000";
    } elsif (bereq.url ~ "\.(woff|woff2|ttf|eot)$") {
        # Cache fonts for 1 year
        set beresp.ttl = 365d;
        set beresp.http.Cache-Control = "public, max-age=31536000, immutable";
        # Add CORS headers for fonts
        set beresp.http.Access-Control-Allow-Origin = "*";
    } elsif (bereq.url ~ "^/api/v1/(raffles|items|categories)$" && bereq.method == "GET") {
        # Cache public API endpoints for 5 minutes
        set beresp.ttl = 5m;
        set beresp.http.Cache-Control = "public, max-age=300";
    } elsif (bereq.url == "/" || bereq.url ~ "\.html$") {
        # Cache HTML for 1 hour
        set beresp.ttl = 1h;
        set beresp.http.Cache-Control = "public, max-age=3600";
    } else {
        # Default: no cache
        set beresp.ttl = 0s;
        set beresp.http.Cache-Control = "no-cache, no-store, must-revalidate";
    }

    # Enable ESI for dynamic content
    if (beresp.http.Content-Type ~ "text/html") {
        set beresp.do_esi = true;
    }

    # Compress responses
    if (beresp.http.Content-Type ~ "text|application/javascript|application/json|application/xml") {
        set beresp.do_gzip = true;
    }

    # Remove server information
    unset beresp.http.Server;
    unset beresp.http.X-Powered-By;

    # Set security headers
    set beresp.http.X-Frame-Options = "SAMEORIGIN";
    set beresp.http.X-Content-Type-Options = "nosniff";
    set beresp.http.X-XSS-Protection = "1; mode=block";
    set beresp.http.Referrer-Policy = "strict-origin-when-cross-origin";

    # Handle errors
    if (beresp.status >= 400) {
        set beresp.ttl = 0s;
        set beresp.uncacheable = true;
        return (deliver);
    }

    return (deliver);
}

# Deliver routine
sub vcl_deliver {
    # Add cache status header
    if (obj.hits > 0) {
        set resp.http.X-Cache = "HIT";
        set resp.http.X-Cache-Hits = obj.hits;
    } else {
        set resp.http.X-Cache = "MISS";
    }

    # Add cache age
    set resp.http.X-Cache-Age = obj.age;

    # Remove internal headers
    unset resp.http.Via;
    unset resp.http.X-Varnish;

    # Add CORS headers for API responses
    if (req.url ~ "^/api/") {
        set resp.http.Access-Control-Allow-Origin = "https://raffleplatform.com";
        set resp.http.Access-Control-Allow-Methods = "GET, POST, PUT, DELETE, OPTIONS";
        set resp.http.Access-Control-Allow-Headers = "Content-Type, Authorization, X-Requested-With";
        set resp.http.Access-Control-Max-Age = "86400";
    }

    # Handle CORS preflight
    if (req.method == "OPTIONS") {
        set resp.status = 200;
        set resp.http.Content-Length = "0";
        set resp.http.Access-Control-Allow-Origin = "*";
        set resp.http.Access-Control-Allow-Methods = "GET, POST, PUT, DELETE, OPTIONS";
        set resp.http.Access-Control-Allow-Headers = "Content-Type, Authorization, X-Requested-With";
        set resp.http.Access-Control-Max-Age = "86400";
        return (deliver);
    }

    return (deliver);
}

# Error handling
sub vcl_backend_error {
    # Custom error page
    set beresp.http.Content-Type = "text/html; charset=utf-8";
    set beresp.status = 503;
    synthetic({"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Service Temporarily Unavailable</title>
            <style>
                body { font-family: Arial, sans-serif; text-align: center; padding: 50px; }
                .error { color: #e74c3c; }
                .message { margin: 20px 0; }
            </style>
        </head>
        <body>
            <h1 class="error">Service Temporarily Unavailable</h1>
            <p class="message">We're experiencing technical difficulties. Please try again in a few moments.</p>
            <p>Error: "} + beresp.reason + {" at "} + now + {"</p>
        </body>
        </html>
    "});
    return (deliver);
}

# Synthetic response for errors
sub vcl_synth {
    if (resp.status == 405) {
        set resp.http.Content-Type = "text/html; charset=utf-8";
        synthetic({"
            <!DOCTYPE html>
            <html>
            <head><title>Method Not Allowed</title></head>
            <body>
                <h1>405 Method Not Allowed</h1>
                <p>The requested method is not allowed for this resource.</p>
            </body>
            </html>
        "});
        return (deliver);
    }

    return (deliver);
}