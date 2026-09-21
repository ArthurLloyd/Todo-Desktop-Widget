import { useRef } from "react";
import { useI18n } from "../lib/i18n";
import type { ThemeCssVariables } from "../lib/themeRuntime";
import type { Task, TaskList, WidgetPane } from "../lib/types";
import { classifyPanePresentation } from "../lib/widgetPresentation";
import { TaskInput } from "./TaskInput";
import { TaskList as TaskListView } from "./TaskList";

type Props = {
  pane: WidgetPane;
  list: TaskList | null;
  lists: TaskList[];
  tasks: Task[];
  completingIds: Set<string>;
  themeStyle?: ThemeCssVariables;
  onSelectList: (paneId: string, listId: string) => void;
  onAdd: (pane: WidgetPane, title: string) => Promise<void>;
  onComplete: (taskId: string) => void;
  onOpenDetails: (task: Task) => void;
};

export function WidgetPaneView({
  pane,
  list,
  lists,
  tasks,
  completingIds,
  themeStyle,
  onSelectList,
  onAdd,
  onComplete,
  onOpenDetails,
}: Props) {
  const { t } = useI18n();
  const taskInputRef = useRef<HTMLInputElement>(null);
  const presentation = classifyPanePresentation({
    paneListId: pane.listId,
    listExists: Boolean(list),
    taskCount: tasks.length,
  });
  const listMissing = presentation === "missing-list";
  const title = pane.titleOverride?.trim() ||
    (list ? displayListName(list, t) : listMissing ? t("pane.listUnavailable") : t("pane.selectList"));

  return (
    <section
      className="widget-pane"
      data-pane-id={pane.id}
      data-theme-override={themeStyle ? "true" : undefined}
      style={themeStyle}
      aria-label={title}
    >
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
      </header>

      {!list ? (
        <div className="pane-empty-state">
          <span>{listMissing ? t("pane.listUnavailableHelp") : t("pane.selectListHelp")}</span>
        </div>
      ) : (
        <>
          <TaskInput inputRef={taskInputRef} onAdd={(title) => onAdd(pane, title)} />
          <TaskListView
            tasks={tasks}
            completingIds={completingIds}
            onComplete={onComplete}
            onOpenDetails={onOpenDetails}
            onAddTask={() => taskInputRef.current?.focus()}
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
