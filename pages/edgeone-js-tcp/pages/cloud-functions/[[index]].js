const pages = require(__dirname + "/included_files/pages.node");
const { Readable } = require('node:stream');
const { connect } = require('node:net');
const http = require('node:http');

export default function onRequest(context) {
    console.log(context);
    console.log(pages);
    return new Promise((resolve, reject) => {
        const request = context.request.clone();
        let req = http.request(request.url, {
            protocol: "http:",
            method: request.method,
            headers: request.headers,
            createConnection: () => connect(pages),
        }, (res) => resolve(new Response(Readable.toWeb(res), {
            status: res.statusCode,
            statusText: res.statusMessage,
            headers: res.headers,
        })));

        request.body ? Readable.fromWeb(request.body).pipe(req) : req.end();
    });
}

