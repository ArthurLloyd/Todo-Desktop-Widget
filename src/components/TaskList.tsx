import type { Task } from "../lib/types";
import { useI18n } from "../lib/i18n";
import { TaskItem } from "./TaskItem";

type Props = {
  tasks: Task[];
  completingIds: Set<string>;
  onComplete: (taskId: string) => void;
  onOpenDetails: (task: Task) => void;
  unavailable?: boolean;
  onAddTask?: () => void;
};

export function TaskList({ tasks, completingIds, onComplete, onOpenDetails, unavailable, onAddTask }: Props) {
  const { t } = useI18n();

  if (unavailable) {
    return <div className="empty-state"><span>{t("widget.listUnavailable")}</span></div>;
  }

  if (tasks.length === 0) {
    return (
      <div className="empty-state">
        <span>{t("task.empty")}</span>
        {onAddTask ? (
          <button className="empty-state__action" type="button" onClick={onAddTask}>
            {t("task.add")}
          </button>
        ) : null}
      </div>
    );
  }

  return (
    <ul className="task-list">
      {tasks.map((task) => (
        <TaskItem
          key={task.id}
          task={task}
          completing={completingIds.has(task.id)}
          onComplete={onComplete}
          onOpenDetails={onOpenDetails}
        />
      ))}
    </ul>
  );
}
