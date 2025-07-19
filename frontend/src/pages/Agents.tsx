import { useEffect, useState } from 'react';
import Sidebar from '../components/Sidebar';

interface Agent {
  id: string;
  hostname: string;
  version: string;
  cpu_load: [number, number, number];
  mem_used_mb: number;
  disk_used_mb: number;
  net_rx_kb: number;
  alert: boolean;
  last_seen: string;
  active: boolean;
}

import { FiRefreshCw } from 'react-icons/fi';

const Agents = () => {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);

  const killAgent = async (id: string) => {
    try {
      const response = await fetch(`http://localhost:3000/api/agents/kill/${id}`, {
        method: 'POST',
      });
      if (response.ok) {
        fetchAgents();
      } else {
        console.error('Failed to kill agent:', await response.text());
      }
    } catch (error) {
      console.error('Failed to kill agent:', error);
    }
  };

  const fetchAgents = async () => {
    setLoading(true);
    try {
      const response = await fetch('http://localhost:3000/api/agents');
      const data = await response.json();
      setAgents(data);
    } catch (error) {
      console.error('Failed to fetch agents:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchAgents();
  }, []);

  return (
    <div className="flex bg-gray-50 min-h-screen">
      <Sidebar />
      <main className="flex-1 p-8">
        <div className="flex justify-between items-center mb-8">
          <h1 className="text-3xl font-bold text-gray-800">Agents</h1>
          <button
            onClick={fetchAgents}
            className="flex items-center px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors"
          >
            <FiRefreshCw className="mr-2" />
            Refresh
          </button>
        </div>
        <div className="bg-white shadow-md rounded-lg overflow-hidden">
          {loading ? (
            <div className="p-8 text-center">Loading agents...</div>
          ) : agents.length === 0 ? (
            <div className="p-8 text-center">No agents found.</div>
          ) : (
            <table className="min-w-full">
              <thead>
                <tr className="bg-gray-200 text-gray-600 uppercase text-sm leading-normal">
                  <th className="py-3 px-6 text-left">Status</th>
                  <th className="py-3 px-6 text-left">ID</th>
                  <th className="py-3 px-6 text-left">Hostname</th>
                  <th className="py-3 px-6 text-left">Version</th>
                  <th className="py-3 px-6 text-center">CPU Load</th>
                  <th className="py-3 px-6 text-center">Memory</th>
                  <th className="py-3 px-6 text-center">Disk</th>
                  <th className="py-3 px-6 text-center">Network RX</th>
                  <th className="py-3 px-6 text-center">Alert</th>
                  <th className="py-3 px-6 text-left">Last Seen</th>
                  <th className="py-3 px-6 text-center">Actions</th>
                </tr>
              </thead>
              <tbody className="text-gray-600 text-sm font-light">
                {agents.map((agent) => (
                  <tr key={agent.id} className="border-b border-gray-200 hover:bg-gray-100">
                    <td className="py-3 px-6 text-left">
                      <span
                        className={`px-2 py-1 rounded-full text-xs ${
                          agent.active ? 'bg-green-200 text-green-800' : 'bg-red-200 text-red-800'
                        }`}
                      >
                        {agent.active ? 'Active' : 'Inactive'}
                      </span>
                    </td>
                    <td className="py-3 px-6 text-left whitespace-nowrap">{agent.id}</td>
                    <td className="py-3 px-6 text-left">{agent.hostname}</td>
                    <td className="py-3 px-6 text-left">{agent.version}</td>
                    <td className="py-3 px-6 text-center">{agent.cpu_load.join(', ')}</td>
                    <td className="py-3 px-6 text-center">{agent.mem_used_mb} MB</td>
                    <td className="py-3 px-6 text-center">{agent.disk_used_mb} MB</td>
                    <td className="py-3 px-6 text-center">{agent.net_rx_kb} KB</td>
                    <td className="py-3 px-6 text-center">{agent.alert ? 'Yes' : 'No'}</td>
                    <td className="py-3 px-6 text-left">{agent.last_seen}</td>
                    <td className="py-3 px-6 text-center">
                      {agent.active && (
                        <button
                          onClick={() => killAgent(agent.id)}
                          className="bg-red-500 text-white py-1 px-3 rounded-lg hover:bg-red-600 transition-colors"
                        >
                          Kill
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </main>
    </div>
  );
};

export default Agents;