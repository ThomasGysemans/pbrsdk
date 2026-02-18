import PocketBase from "pocketbase";
import assert from "node:assert";
import demo from "./demo-data.json" with { type: 'json' };

const pb = new PocketBase("http://localhost:8091/");

async function main() {
    await pb.collection("_superusers").authWithPassword("thomas@gysemans.dev", "thomasgysemans");
    const collections = (await pb.collections.getFullList()).filter(c => !c.name.startsWith("_"));
    const collectionNames = collections.map(c => c.name);
    const demoCollections = Object.keys(demo.existingCollections);
    assert.ok(collectionNames.every(c => demoCollections.includes(c)));
    assert.ok(demoCollections.every(c => collectionNames.includes(c)));
    for (const collection of collections) {
        const pbFields = collection.fields.filter(f => !f.hidden).map(f => f.name);
        const demoFields = demo.existingCollections[collection.name].fields;
        assert.ok(pbFields.every(c => demoFields.includes(c)));
        assert.ok(demoFields.every(c => pbFields.includes(c)));
    }
    await Promise.all(collectionNames.map(name => pb.collections.truncate(name)));
    for (const collectionName of Object.keys(demo.data)) {
        const records = demo.data[collectionName];
        for (const record of records) {
            await pb.collection(collectionName).create(record);
        }
    }
}

try {
    console.log("Working...");
    await main();
    const cookie = pb.authStore.exportToCookie();
    const parsed = cookieParse(cookie);
    console.log(cookie);
    console.log(parsed);
    pb.authStore.clear();
    console.log("Done.");
} catch (e) {
    console.error("Failed to login as super user :");
    console.error(e);
}


function cookieParse(str) {
    const result = {};

    if (typeof str !== "string") {
        return result;
    }

    let index = 0;
    while (index < str.length) {
        const eqIdx = str.indexOf("=", index);

        // no more cookie pairs
        if (eqIdx === -1) {
            break;
        }

        let endIdx = str.indexOf(";", index);

        if (endIdx === -1) {
            endIdx = str.length;
        } else if (endIdx < eqIdx) {
            // backtrack on prior semicolon
            index = str.lastIndexOf(";", eqIdx - 1) + 1;
            continue;
        }

        const key = str.slice(index, eqIdx).trim();

        // only assign once
        if (undefined === result[key]) {
            let val = str.slice(eqIdx + 1, endIdx).trim();

            // quoted values
            if (val.charCodeAt(0) === 0x22) {
                val = val.slice(1, -1);
            }

            try {
                result[key] = defaultDecode(val);
            } catch (_) {
                result[key] = val; // no decoding
            }
        }

        index = endIdx + 1;
    }

    return result;
}

function defaultDecode(val) {
    return val.indexOf("%") !== -1 ? decodeURIComponent(val) : val;
}