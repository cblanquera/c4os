describe("QA shell fixture gate", () => {
  afterEach(() => {
    vi.unstubAllEnvs();
    vi.resetModules();
  });

  it("returns no fixture state in an ordinary renderer build", async () => {
    vi.stubEnv("VITE_C4OS_QA_FIXTURES", "0");
    const { createBuildGatedQaPreloadedState } = await import("./qa-fixtures");

    expect(createBuildGatedQaPreloadedState()).toBeUndefined();
  });

  it("returns the same deterministic state only in an explicit QA build", async () => {
    vi.stubEnv("VITE_C4OS_QA_FIXTURES", "1");
    const { createBuildGatedQaPreloadedState } = await import("./qa-fixtures");

    const first = createBuildGatedQaPreloadedState();
    const second = createBuildGatedQaPreloadedState();

    expect(first).toEqual(second);
    expect(first?.shellQa).toEqual({
      enabled: true,
      fixtureId: "r013-shell-v1",
      activeWorkflow: "chat",
    });
    expect(first?.shellAuthority?.workspace.value.displayName).toBe(
      "C4OS QA Workspace",
    );
    expect(first?.shellDrafts?.composer.text).toBe(
      "Preserved QA composer draft",
    );
  });
});
