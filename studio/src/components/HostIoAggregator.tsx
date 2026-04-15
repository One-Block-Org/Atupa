import type { AggregatedHostIO } from '../types/trace';
import { fmtEquiv } from '../types/trace';

interface Props {
  rows: AggregatedHostIO[];
}

export function HostIoAggregator({ rows }: Props) {
  if (rows.length === 0) {
    return (
      <div style={{ color: 'var(--color-text-muted)', fontSize: 13 }}>
        No Stylus HostIO calls in this trace.
      </div>
    );
  }

  const maxCost = rows[0]?.total_cost_equiv ?? 1;

  return (
    <table className="hostio-table" aria-label="Host IO aggregator">
      <thead>
        <tr>
          <th>HostIO Name</th>
          <th style={{ textAlign: 'right' }}>Calls</th>
          <th style={{ textAlign: 'right' }}>Gas-equiv</th>
          <th style={{ textAlign: 'right' }}>%</th>
          <th className="flame-bar-cell">Distribution</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <tr key={row.name}>
            <td className="hostio-name">{row.name}</td>
            <td className="hostio-pct">{row.call_count}</td>
            <td className="hostio-gas">{fmtEquiv(row.total_cost_equiv)}</td>
            <td className="hostio-pct">{row.pct.toFixed(1)}%</td>
            <td className="flame-bar-cell">
              <div className="flame-bar-track">
                <div
                  className="flame-bar-fill"
                  style={{ width: `${(row.total_cost_equiv / maxCost) * 100}%` }}
                />
              </div>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
