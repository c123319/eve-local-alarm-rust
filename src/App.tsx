import { useEffect } from 'react';
import SettingsPanel from './components/SettingsPanel';

function App() {
  // Set document title to Chinese on mount
  useEffect(() => {
    document.title = 'EVE 本地警报';
  }, []);

  return (
    <div className="app">
      <SettingsPanel />
    </div>
  );
}

export default App;
