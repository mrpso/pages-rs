const pages = require(__dirname + "/included_files/pages.node");

export default async function onRequest(context) {
    console.log(context);

    const request = context.request;
    const response = await pages.http({
        method: request.method,
        url: request.url,
        headers: Object.fromEntries(request.headers),
        body: new Buffer(await request.arrayBuffer())
    })

    return new Response(response.body, {
        status: response.status,
        headers: response.headers,
    });
}

