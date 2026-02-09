import React, { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import axios from 'axios';
import { format } from 'date-fns';

function BlockDetail() {
  const { id } = useParams();
  const [block, setBlock] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    fetchBlock();
  }, [id]);

  const fetchBlock = async () => {
    try {
      const response = await axios.get(`/api/block/${id}`);
      setBlock(response.data);
      setLoading(false);
    } catch (err) {
      setError(err.message);
      setLoading(false);
    }
  };

  const formatAXM = (satoshis) => {
    return (satoshis / 100000000).toFixed(8);
  };

  const shortenHash = (hash) => {
    return `${hash.substring(0, 12)}...${hash.substring(hash.length - 12)}`;
  };

  if (loading) return <div className="loading">Loading block...</div>;
  if (error) return <div className="error">Error: {error}</div>;
  if (!block) return <div className="error">Block not found</div>;

  return (
    <div>
      <div className="card">
        <h2 style={{ marginBottom: '30px' }}>Block #{block.index}</h2>
        
        <div className="detail-row">
          <div className="detail-label">Hash:</div>
          <div className="detail-value hash">{block.hash}</div>
        </div>

        <div className="detail-row">
          <div className="detail-label">Previous Hash:</div>
          <div className="detail-value">
            {block.index > 0 ? (
              <Link to={`/block/${block.index - 1}`} className="hash">
                {block.previous_hash}
              </Link>
            ) : (
              <span className="hash">{block.previous_hash}</span>
            )}
          </div>
        </div>

        <div className="detail-row">
          <div className="detail-label">Timestamp:</div>
          <div className="detail-value">
            {format(new Date(block.timestamp * 1000), 'PPpp')}
          </div>
        </div>

        <div className="detail-row">
          <div className="detail-label">Miner:</div>
          <div className="detail-value">
            <Link to={`/address/${block.miner}`} className="hash">
              {block.miner}
            </Link>
          </div>
        </div>

        <div className="detail-row">
          <div className="detail-label">Transactions:</div>
          <div className="detail-value">
            <span className="badge badge-info">{block.transactions.length}</span>
          </div>
        </div>

        <div className="detail-row">
          <div className="detail-label">Difficulty:</div>
          <div className="detail-value">{block.difficulty.toLocaleString()}</div>
        </div>

        <div className="detail-row">
          <div className="detail-label">Nonce:</div>
          <div className="detail-value">{block.nonce}</div>
        </div>

        <div className="detail-row">
          <div className="detail-label">Merkle Root:</div>
          <div className="detail-value hash">{block.merkle_root}</div>
        </div>

        {block.vdf_output && (
          <div className="detail-row">
            <div className="detail-label">VDF Output:</div>
            <div className="detail-value hash">{block.vdf_output}</div>
          </div>
        )}

        <div className="detail-row">
          <div className="detail-label">Size:</div>
          <div className="detail-value">{(block.size / 1024).toFixed(2)} KB</div>
        </div>

        <div className="detail-row">
          <div className="detail-label">Block Reward:</div>
          <div className="detail-value">{formatAXM(block.reward)} AXM</div>
        </div>
      </div>

      {block.transactions.length > 0 && (
        <div className="card">
          <h3 style={{ marginBottom: '20px' }}>Transactions</h3>
          <table className="table">
            <thead>
              <tr>
                <th>Hash</th>
                <th>From</th>
                <th>To</th>
                <th>Amount</th>
                <th>Fee</th>
                <th>Privacy</th>
              </tr>
            </thead>
            <tbody>
              {block.transactions.map((tx) => (
                <tr key={tx.hash}>
                  <td>
                    <Link to={`/transaction/${tx.hash}`} className="hash-short">
                      {shortenHash(tx.hash)}
                    </Link>
                  </td>
                  <td>
                    <Link to={`/address/${tx.sender}`} className="hash-short">
                      {shortenHash(tx.sender)}
                    </Link>
                  </td>
                  <td>
                    <Link to={`/address/${tx.recipient}`} className="hash-short">
                      {shortenHash(tx.recipient)}
                    </Link>
                  </td>
                  <td>{formatAXM(tx.amount)} AXM</td>
                  <td>{formatAXM(tx.fee)} AXM</td>
                  <td>
                    {tx.zk_proof ? (
                      <span className="badge badge-success">ZK-STARK</span>
                    ) : (
                      <span className="badge badge-warning">Public</span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

export default BlockDetail;
