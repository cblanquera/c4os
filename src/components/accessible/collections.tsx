import type { ReactNode } from "react";
import type {
  DisclosureProps as AriaDisclosureProps,
  TabsProps as AriaTabsProps,
} from "react-aria-components";
import {
  Button as AriaButton,
  Disclosure as AriaDisclosure,
  DisclosurePanel,
  Heading,
  Tab,
  TabList,
  TabPanel,
  Tabs as AriaTabs,
} from "react-aria-components";

import { Icon } from "./icons";

export interface DisclosureProps extends Omit<
  AriaDisclosureProps,
  "children" | "className"
> {
  readonly children: ReactNode;
  readonly title: ReactNode;
}

export interface TabDefinition {
  readonly content: ReactNode;
  readonly id: string;
  readonly label: ReactNode;
}

export interface TabsProps extends Omit<
  AriaTabsProps,
  "children" | "className"
> {
  readonly label: string;
  readonly tabs: readonly TabDefinition[];
}

export interface ScanRowProps {
  readonly actions?: ReactNode;
  readonly actionsLabel?: string;
  readonly className?: string;
  readonly description?: ReactNode;
  readonly leading?: ReactNode;
  readonly metadata?: ReactNode;
  readonly title: ReactNode;
}

/** Renders a keyboard-operable disclosure with connected trigger and panel. */
export function Disclosure({ children, title, ...props }: DisclosureProps) {
  return (
    <AriaDisclosure {...props} className="c4-disclosure">
      <Heading className="c4-disclosure__heading">
        <AriaButton className="c4-disclosure__trigger" slot="trigger">
          <Icon className="c4-disclosure__chevron" name="chevron-right" />
          <span>{title}</span>
        </AriaButton>
      </Heading>
      <DisclosurePanel className="c4-disclosure__panel">
        {children}
      </DisclosurePanel>
    </AriaDisclosure>
  );
}

/** Renders connected, arrow-key navigable tabs and their selected panel. */
export function Tabs({ label, tabs, ...props }: TabsProps) {
  return (
    <AriaTabs {...props} className="c4-tabs">
      <TabList aria-label={label} className="c4-tab-list">
        {tabs.map((tab) => (
          <Tab className="c4-tab" id={tab.id} key={tab.id}>
            {tab.label}
          </Tab>
        ))}
      </TabList>
      {tabs.map((tab) => (
        <TabPanel className="c4-tab-panel" id={tab.id} key={tab.id}>
          {tab.content}
        </TabPanel>
      ))}
    </AriaTabs>
  );
}

/** Renders a dense, scan-oriented row with permanently reserved action space. */
export function ScanRow({
  actions,
  actionsLabel = "Row actions",
  className,
  description,
  leading,
  metadata,
  title,
}: ScanRowProps) {
  return (
    <article
      className={["c4-scan-row", className].filter(Boolean).join(" ")}
      data-layout="reserved-actions"
    >
      {leading ? <div className="c4-scan-row__leading">{leading}</div> : null}
      <div className="c4-scan-row__copy">
        <h3>{title}</h3>
        {description ? <p>{description}</p> : null}
      </div>
      {metadata ? (
        <div className="c4-scan-row__metadata">{metadata}</div>
      ) : null}
      {actions ? (
        <div aria-label={actionsLabel} className="c4-scan-row__actions">
          {actions}
        </div>
      ) : null}
    </article>
  );
}
