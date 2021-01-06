const napi = require('./index.node')

async function main() {
    const e = new napi.SqlEngine("server=tcp:localhost,1433;user=SA;password=<YourStrong@Passw0rd>;TrustServerCertificate=true;encrypt=DANGER_PLAINTEXT");
    let res = await e.select_1();

    return res
}

main().then((res) => console.log(res))
