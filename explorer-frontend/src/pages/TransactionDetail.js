import React, { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import axios from 'axios';
import { format } from 'date-fns';

function TransactionDetail() {
  const { hash } = useParams();
  const [tx, setTx] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    fetchTransaction();
  }, [hash]);

  const fetchTransaction = async () => {
    try {
      const response = await axios.get(`/api/transaction/${hash}`);
      setTx(response.data);
      setLoading(false);
    } catch (err) {
      setError(err.message);
      setLoading(false);
    }
  };

  const formatAXM = (satoshis) => {
    return (satoshis / 100000000).toFixed(8);
  };

  if (loading) return <div className="loading">Loading transaction...</div>;
  if (error) return <div className="error">Error: {error}</div>;
  if (!tx) return <div className="error">Transaction not found</div>;

  return (
    <div className="card">
      <h2 style={{ marginBottom: '30px' }}>Transaction Details</h2>
      
      <div className="detail-row">
        <div className="detail-label">Hash:</div>
        <div className="detail-value hash">{tx.hash}</div>
      </div>

      <div className="detail-row">
        <div className="detail-label">Status:</div>
        <div className="detail-value">
          {tx.block_index ? (
            <span className="badge badge-success">Confirmed ({tx.confirmations} confirmations)</span>
          ) : (
            <span className="badge badge-warning">Pending</span>
          )}
        </div>
      </div>

      {tx.block_hash && (
        <div className="detail-row">
          <div className="detail-label">Block:</div>
          <div className="detail-value">
            <Link to={`/block/${tx.block_index}`}>
              Block #{tx.block_index}
            </Link>
          </div>
        </div>
      )}

      <div className="detail-row">
        <div className="detail-label">Timestamp:</div>
        <div className="detail-value">
          {format(new Date(tx.timestamp * 1000), 'PPpp')}
        </div>
      </div>

      <div className="detail-row">
        <div className="detail-label">From:</div>
        <div className="detail-value">
          <Link to={`/address/${tx.sender}`} className="hash">
            {tx.sender}
          </Link>
        </div>
      </div>

      <div className="detail-row">
        <div className="detail-label">To:</div>
        <div className="detail-value">
          <Link to={`/address/${tx.recipient}`} className="hash">
            {tx.recipient}
          </Link>
        </div>
      </div>

      <div className="detail-row">
        <div className="detail-label">Amount:</div>
        <div className="detail-value">
          <strong style={{ fontSize: '20px', color: '#667eea' }}>
            {formatAXM(tx.amount)} AXM
          </strong>
        </div>
      </div>

      <div className="detail-row">
        <div className="detail-label">Fee:</div>
        <div className="detail-value">{formatAXM(tx.fee)} AXM</div>
      </div>

      <div className="detail-row">
        <div className="detail-label">Privacy:</div>
        <div className="detail-value">
          {tx.zk_proof ? (
            <>
              <span className="badge badge-success">ZK-STARK Enabled</span>
              <p style={{ marginTop: '10px', fontSize: '14px', color: '#666' }}>
                This transaction uses zero-knowledge proofs for enhanced privacy
              </p>
            </>
          ) : (
            <span className="badge badge-warning">Public Transaction</span>
          )}
        </div>
      </div>

      <div className="detail-row">
        <div className="detail-label">Signature:</div>
        <div className="detail-value hash" style={{ fontSize: '12px' }}>
          {tx.signature}
        </div>
      </div>

      {tx.zk_proof && (
        <div className="detail-row">
          <div className="detail-label">ZK Proof:</div>
          <div className="detail-value hash" style={{ fontSize: '12px' }}>
            {tx.zk_proof}
          </div>
        </div>
      )}
    </div>
  );
}

export default TransactionDetail;
