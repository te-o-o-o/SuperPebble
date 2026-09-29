import { useCallback, useEffect, useRef, useState } from "react";
import { ReactFlowProvider } from "@xyflow/react";
import { api } from "./api";
import { t } from "./i18n";
import { AccountsDialog } from "./components/AccountsDialog";
import { BranchPanel } from "./components/BranchPanel";
import { DetailPanel } from "./components/DetailPanel";
import { GraphView } from "./components/GraphView";
import { IssueBar } from "./components/IssueBar";
import { Sidebar } from "./components/Sidebar";
import { TopBar } from "./components/TopBar";
import { TransferDialog } from "./components/TransferDialog";
import type { GroupKey } from "./theme";
import type { Account, Graph, PNode, Scope } from "./types";

const toggled = <T,>(set: Set<T>, v: T) => {
  const next = new Set(set);
  next.has(v) ? next.delete(v) : next.add(v);
  return next;
};

export default function App() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [account, setAccount] = useState("");
  const [projects, setProjects] = useState<string[]>([]);
  const [project, setProject] = useState<string | null>(null);
  const [graph, setGraph] = useState<Graph>();
  const [error, setError] = useState<string>();
  const [hidden, setHidden] = useState(new Set<GroupKey>());
  const [expanded, setExpanded] = useState(new Set<string>());
  const [selected, setSelected] = useState<string | null>(null);
  const [scope, setScope] = useState<Scope | null>(null);
  const [accountsOpen, setAccountsOpen] = useState(false);
  const [moving, setMoving] = useState<PNode | null>(null);

  useEffect(() => {
    api.accounts().then((a) => {
      setAccounts(a);
      setAccount(a.find((x) => x.is_default)?.config_dir ?? a[0]?.config_dir ?? "");
    });
  }, []);

  useEffect(() => {
    if (!account) return;
    api.projects(account).then((p) => {
      setProjects(p);
      setProject((cur) => (cur && p.includes(cur) ? cur : null));
    });
  }, [account]);

  const rescan = useCallback(() => {
    if (!account) return;
    api.scan(account, project).then(
      (g) => {
        setGraph(g);
        setError(undefined);
      },
      (e) => setError(t(String(e))),
    );
  }, [account, project]);

  // Scan now, then again (debounced) whenever a watched file changes.
  const debounce = useRef<number>(undefined);
  useEffect(() => {
    rescan();
    const unlisten = api.watch(account, project, () => {
      clearTimeout(debounce.current);
      debounce.current = window.setTimeout(rescan, 300);
    });
    return () => void unlisten.then((f) => f());
  }, [rescan, account, project]);

  const node = graph?.nodes.find((n) => n.id === selected);
  const readOnly = accounts.find((a) => a.config_dir === account)?.wsl;

  return (
    <div className="app">
      <TopBar
        accounts={accounts}
        account={account}
        onAccount={setAccount}
        projects={projects}
        project={project}
        onProject={setProject}
        scannedAt={graph?.scanned_at}
        onRescan={rescan}
      />
      <Sidebar
        graph={graph}
        hidden={hidden}
        onToggle={(k) => setHidden((h) => toggled(h, k))}
        scope={scope}
        onAccounts={() => setAccountsOpen(true)}
        onScope={(s) => {
          setScope(s);
          setSelected(null);
        }}
      />
      <ReactFlowProvider>
        {graph ? (
          <GraphView
            graph={graph}
            hidden={hidden}
            expanded={expanded}
            selected={selected}
            scope={scope}
            onSelect={setSelected}
            onToggle={(k) => setExpanded((e) => toggled(e, k))}
          />
        ) : (
          <div className="canvas center muted">{error ?? "Scan…"}</div>
        )}
      </ReactFlowProvider>
      {graph &&
        (selected?.startsWith("group:") ? (
          <BranchPanel graph={graph} group={selected.slice(6) as GroupKey} onSelect={setSelected} />
        ) : (
          <DetailPanel graph={graph} node={node} onOpen={api.openPath} onSelect={setSelected} onMove={readOnly ? undefined : setMoving} />
        ))}
      <AccountsDialog open={accountsOpen} accounts={accounts} onClose={() => setAccountsOpen(false)} onChange={setAccounts} />
      <TransferDialog node={moving} accounts={accounts} account={account} project={project} onClose={() => setMoving(null)} onDone={rescan} />
      <IssueBar issues={graph?.issues ?? []} onSelect={setSelected} />
    </div>
  );
}
