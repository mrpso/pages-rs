const pages = require(__dirname + "/included_files/pages.node");
const { fetch, buildConnector, Agent } = require('undici');

const connector = buildConnector({
    rejectUnauthorized: false,
    socketPath: pages.path,
});
const agent = new Agent({
    connect: (_, callback) => {
        const options = {
            protocol: "http:",
            hostname: pages.host,
            port: pages.port,
        };
        connector(options, callback);
    },
});

export default async function onRequest(context) {
    const request = context.request.clone();
    const rep = await fetch(request.url, {
        dispatcher: agent,
        method: request.method,
        headers: request.headers,
        body: request.body || undefined,
        duplex: request.body ? 'half' : undefined,
    });

    return new Response(rep.body, {
        status: rep.status,
        headers: rep.headers,
        statusText: rep.statusText,
    });
}

