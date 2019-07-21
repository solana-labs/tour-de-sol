const fs = require('fs');
const path = require('path');
const yaml = require('yaml');
const IPGeolocationAPI = require('ip-geolocation-api-javascript-sdk');
const GeolocationParams = require('ip-geolocation-api-javascript-sdk/GeolocationParams.js');

const {GEOLOCATION_API_KEY} = process.env;
if (typeof GEOLOCATION_API_KEY !== 'string') {
  console.error('GEOLOCATION_API_KEY not found in the environment');
  console.error('Obtain an API key from https://ipgeolocation.io');
  process.exit(1);
}
const ipgeolocationApi = new IPGeolocationAPI(GEOLOCATION_API_KEY, false);

function logError(err) {
  console.log(err.message);
}

let observed = {};
try {
  observed = JSON.parse(fs.readFileSync('observed-ip-addresses.json', 'utf8'));
  console.log('Loaded observed-ip-addresses.json');
} catch (err) {
  logError(err);
}

let earthValidators = [];
try {
  earthValidators = Object.keys(
    yaml.parse(
      fs.readFileSync(path.join(__dirname, '..', 'validators', 'earth.yml'), 'utf8')
    ) || []
  )
} catch (err) {
  logError(err);
}

for (const pubkey in observed) {
  if (earthValidators.includes(pubkey)) {
    for (const ip of observed[pubkey]) {
      const geolocationParams = new GeolocationParams();
      geolocationParams.setLang('en');
      geolocationParams.setIPAddress(ip);
      ipgeolocationApi.getGeolocation(
        json => {
            if (json.country_name === 'United States') {
              console.log(`Error: Validator ${pubkey} observed with US address ${ip}`);
            }
        },
        geolocationParams
      );
    }
  }
}


