import { useEffect, useState } from "react";
import { Button } from "react-aria-components";

import type { FoundationSnapshot } from "../../platform/protocol";
import { readFoundationSnapshot } from "../../platform/foundation";

interface FoundationScreenProps {
  readonly loadSnapshot?: () => Promise<FoundationSnapshot>;
}

type CoreConnection =
  | { readonly status: "loading" }
  | { readonly status: "connected"; readonly snapshot: FoundationSnapshot }
  | { readonly status: "unavailable" };

export function FoundationScreen({
  loadSnapshot = readFoundationSnapshot,
}: FoundationScreenProps) {
  const [connection, setConnection] = useState<CoreConnection>({
    status: "loading",
  });

  useEffect(() => {
    let active = true;

    void loadSnapshot().then(
      (snapshot) => {
        if (active) {
          setConnection({ status: "connected", snapshot });
        }
      },
      () => {
        if (active) {
          setConnection({ status: "unavailable" });
        }
      },
    );

    return () => {
      active = false;
    };
  }, [loadSnapshot]);

  const protocol =
    connection.status === "connected"
      ? `Version ${connection.snapshot.protocolVersion} · fail closed`
      : "Version pending · fail closed";
  const durableAuthority =
    connection.status === "connected"
      ? "Connected · Rust core"
      : connection.status === "loading"
        ? "Connecting…"
        : "Unavailable · fail closed";

  return (
    <main className="foundation" data-testid="foundation-screen">
      <section className="foundation__card" aria-labelledby="foundation-title">
        <div className="foundation__mark" aria-hidden="true">
          C4
        </div>
        <p className="foundation__eyebrow">Local macOS development build</p>
        <h1 id="foundation-title">C4OS foundation is running</h1>
        <p>
          The production shell uses a versioned, Rust-owned boundary. Workspace
          and Chat capabilities arrive in the next verified tasks.
        </p>
        <dl className="foundation__status">
          <div>
            <dt>Protocol</dt>
            <dd>{protocol}</dd>
          </div>
          <div>
            <dt>Renderer</dt>
            <dd>Projection only</dd>
          </div>
          <div>
            <dt>Durable authority</dt>
            <dd role="status">{durableAuthority}</dd>
          </div>
        </dl>
        <Button className="foundation__button" isDisabled>
          Start a Workspace
        </Button>
        <p className="foundation__note">
          Unavailable until the Workspace lifecycle is verified.
        </p>
      </section>
    </main>
  );
}
