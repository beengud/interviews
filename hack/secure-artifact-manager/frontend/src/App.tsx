import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import Layout from "./components/Layout";
import Dashboard from "./pages/Dashboard";
import Artifacts from "./pages/Artifacts";
import ScanResults from "./pages/ScanResults";
import AuditLog from "./pages/AuditLog";

function App() {
  return (
    <Router>
      <Layout>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/artifacts" element={<Artifacts />} />
          <Route path="/scan-results" element={<ScanResults />} />
          <Route path="/audit-log" element={<AuditLog />} />
        </Routes>
      </Layout>
    </Router>
  );
}

export default App;