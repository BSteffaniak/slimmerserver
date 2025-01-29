function getCustomDomain() {
    return {
        name: domainName,
        dns: sst.cloudflare.dns(),
    };
}

const isProd = $app.stage === 'prod';
const subdomain = isProd ? '' : `marketing-${$app.stage}.`;
const domain = process.env.DOMAIN ?? 'moosicbox.com';
const domainName = `${subdomain}${domain}`;

const customDomain = getCustomDomain();

const site = new sst.aws.StaticSite('MoosicBoxMarketingSite', {
    build: {
        command: "cargo run --no-default-features --features htmx,dev gen",
        output: "gen"
    },
    domain: customDomain,
    environment: {},
});

export const outputs = {
    url: site.url,
    host: `https://${domainName}`,
};
