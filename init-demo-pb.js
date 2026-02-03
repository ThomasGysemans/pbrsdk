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
            if (collectionName === "users") {
                record.password = "qwertyui";
                record.passwordConfirm = "qwertyui";
            }
            await pb.collection(collectionName).create(record);
        }
    }
    console.log(pb.authStore.token);
    // console.log(await (await fetch("http://localhost:8091/api/collections", {
    //     headers: {
    //         "Authorization": `Bearer ${pb.authStore.token}`
    //     }
    // })).json());
}

try {
    console.log("Working...");
    await main();
    console.log("Done.");
} catch (e) {
    console.error("Failed to login as super user :");
    console.error(e);
}

pb.authStore.clear();