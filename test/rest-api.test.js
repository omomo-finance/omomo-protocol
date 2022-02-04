
describe("Preparation set", () => {

    test("Environment shoud be `test`", async () => {
        expect(process.env.NODE_ENV).toBe("test");
    });

})