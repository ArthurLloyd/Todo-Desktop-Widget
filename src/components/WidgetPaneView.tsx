import { Trash2 } from "lucide-react";
import { useI18n } from "../lib/i18n";
import type { Task, TaskList, WidgetPane } from "../lib/types";
import { TaskInput } from "./TaskInput";
import { TaskList as TaskListView } from "./TaskList";

type Props = {
  pane: WidgetPane;
  list: TaskList | null;
  lists: TaskList[];
  tasks: Task[];
  completingIds: Set<string>;
  canDelete: boolean;
  onSelectList: (paneId: string, listId: string) => void;
  onAdd: (pane: WidgetPane, title: string) => Promise<void>;
  onComplete: (taskId: string) => void;
  onOpenDetails: (task: Task) => void;
  onDelete: (paneId: string) => void;
};

export function WidgetPaneView({
  pane,
  list,
  lists,
  tasks,
  completingIds,
  canDelete,
  onSelectList,
  onAdd,
  onComplete,
  onOpenDetails,
  onDelete,
}: Props) {
  const { t } = useI18n();
  const listMissing = Boolean(pane.listId && !list);
  const title = pane.titleOverride?.trim() ||
    (list ? displayListName(list, t) : listMissing ? t("pane.listUnavailable") : t("pane.selectList"));

  return (
    <section className="widget-pane" data-pane-id={pane.id} aria-label={title}>
      <header className="widget-pane__header">
        <strong className="widget-pane__title">{title}</strong>
        <select
          className="pane-list-select"
          aria-label={t("pane.listSelector", { name: title })}
          value={list?.id || ""}
          onChange={(event) => {
            if (event.target.value) onSelectList(pane.id, event.target.value);
          }}
        >
          <option value="" disabled>
            {listMissing ? t("pane.listUnavailable") : t("pane.selectList")}
          </option>
          {lists.map((item) => (
            <option key={item.id} value={item.id}>
              {displayListName(item, t)}
            </option>
          ))}
        </select>
        <button
          className="icon-button icon-button--danger"
          type="button"
          disabled={!canDelete}
          title={canDelete ? t("pane.remove") : t("pane.atLeastOne")}
          aria-label={t("pane.removeNamed", { name: title })}
          onClick={() => onDelete(pane.id)}
        >
          <Trash2 size={15} />
        </button>
      </header>

      {!list ? (
        <div className="pane-empty-state">
          <span>{listMissing ? t("pane.listUnavailableHelp") : t("pane.selectListHelp")}</span>
        </div>
      ) : (
        <>
          <TaskInput onAdd={(title) => onAdd(pane, title)} />
          <TaskListView
            tasks={tasks}
            listName={list.displayName}
            listRole={list.role}
            completingIds={completingIds}
            onComplete={onComplete}
            onOpenDetails={onOpenDetails}
          />
        </>
      )}
    </section>
  );
}

function displayListName(list: TaskList, t: ReturnType<typeof useI18n>["t"]) {
  if (list.role === "today") return t("app.today");
  if (list.role === "inbox") return t("app.inbox");
  if (list.role === "this_week") return t("app.thisWeek");
  return list.displayName;
}
