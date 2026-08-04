/**
 * Root application component.
 *
 * Layout: fixed left sidebar + scrollable right content area, once a project
 * is open. Before that, the Welcome screen fills the whole window.
 */

import { useExecutionStore } from './store/executionStore';
import { useAutoSave } from './hooks/useAutoSave';
import { Sidebar } from './components/Sidebar';
import { NotificationTray } from './components/NotificationTray';
import { Welcome } from './screens/Welcome';
import { Dashboard } from './screens/Dashboard';
import { Personnel } from './screens/Personnel';
import { WorkPackages } from './screens/WorkPackages';
import { Milestones } from './screens/Milestones';
import { Deliverables } from './screens/Deliverables';
import { Amendments } from './screens/Amendments';
import { Travel } from './screens/Travel';
import { Equipment } from './screens/Equipment';
import { OtherCosts } from './screens/OtherCosts';
import { Subcontracting } from './screens/Subcontracting';
import { ReportingPeriods } from './screens/ReportingPeriods';
import { RiskRegister } from './screens/RiskRegister';
import { IssueLog } from './screens/IssueLog';
import { ReportsExport } from './screens/ReportsExport';
import './App.css';

export function App() {
  const activeScreen = useExecutionStore((s) => s.activeScreen);

  useAutoSave();

  if (activeScreen === 'welcome') {
    return <Welcome />;
  }

  const renderScreen = () => {
    switch (activeScreen) {
      case 'personnel':
        return <Personnel />;
      case 'work-packages':
        return <WorkPackages />;
      case 'milestones':
        return <Milestones />;
      case 'deliverables':
        return <Deliverables />;
      case 'amendments':
        return <Amendments />;
      case 'travel':
        return <Travel />;
      case 'equipment':
        return <Equipment />;
      case 'other-costs':
        return <OtherCosts />;
      case 'subcontracting':
        return <Subcontracting />;
      case 'reporting-periods':
        return <ReportingPeriods />;
      case 'risk-register':
        return <RiskRegister />;
      case 'issue-log':
        return <IssueLog />;
      case 'reports-export':
        return <ReportsExport />;
      case 'dashboard':
      default:
        return <Dashboard />;
    }
  };

  return (
    <div className="app-shell">
      <Sidebar />
      <main className="content-area">{renderScreen()}</main>
      <NotificationTray />
    </div>
  );
}
