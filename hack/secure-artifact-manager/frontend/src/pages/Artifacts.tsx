import {
  Typography,
  Box,
  Stack,
  TextField,
  Button,
  Paper,
} from "@mui/material";
import {
  DataGrid,
  GridColDef,
  GridRenderCellParams,
} from "@mui/x-data-grid";
import { useEffect, useState, ChangeEvent } from "react";
import axios from "axios";

const API_BASE = import.meta.env.VITE_API_BASE_URL;

type Artifact = {
  id: string;
  original_filename: string;
  size_bytes: number;
  sha256: string;
  upload_time: string;
  metadata: string;
  scan_status: string;
};

function Artifacts() {
  const [artifacts, setArtifacts] = useState<Artifact[]>([]);
  const [file, setFile] = useState<File | null>(null);

  const fetchArtifacts = async () => {
    const res = await axios.get<Artifact[]>(`${API_BASE}/artifacts`);
    setArtifacts(res.data);
  };

  const upload = async () => {
    if (!file) return;
    const formData = new FormData();
    formData.append("file", file);

    await axios.post(`${API_BASE}/upload`, formData, {
      headers: { "Content-Type": "multipart/form-data" },
    });

    setFile(null);
    fetchArtifacts();
  };

  const getScanStatusDisplay = (status: string) => {
    switch (status) {
      case "clean":
        return (
          <Box
            sx={{
              bgcolor: "#d0f0c0",
              borderRadius: "4px",
              px: 1,
              py: 0.5,
              display: "inline-flex",
              alignItems: "center",
              fontWeight: "bold",
            }}
            title="Clean"
          >
            ✅
          </Box>
        );
      case "in-progress":
        return (
          <Box
            sx={{
              bgcolor: "#fff4cc",
              borderRadius: "4px",
              px: 1,
              py: 0.5,
              display: "inline-flex",
              alignItems: "center",
              fontWeight: "bold",
              animation: "spin 2s linear infinite",
              "@keyframes spin": {
                "0%": { transform: "rotate(0deg)" },
                "100%": { transform: "rotate(360deg)" },
              },
            }}
            title="Scanning..."
          >
            🔄
          </Box>
        );
      case "quarantined":
        return (
          <Box
            sx={{
              bgcolor: "#f8d7da",
              borderRadius: "4px",
              px: 1,
              py: 0.5,
              display: "inline-flex",
              alignItems: "center",
              fontWeight: "bold",
            }}
            title="Quarantined"
          >
            🛑
          </Box>
        );
      default:
        return <Box>{status}</Box>;
    }
  };

  const columns: GridColDef[] = [
    { field: "original_filename", headerName: "Name", flex: 2, minWidth: 150 },
    {
      field: "scan_status",
      headerName: "Scan Status",
      flex: 1,
      minWidth: 120,
      renderCell: (params: GridRenderCellParams<Artifact>) =>
        getScanStatusDisplay(params.value),
      sortable: false,
    },
    { field: "metadata", headerName: "Status", flex: 1, minWidth: 120 },
    { field: "size_bytes", headerName: "Size (bytes)", flex: 1, minWidth: 100 },
    { field: "upload_time", headerName: "Uploaded At", flex: 2, minWidth: 180 },
    {
      field: "download",
      headerName: "Actions",
      sortable: false,
      minWidth: 130,
      renderCell: (params: GridRenderCellParams<Artifact>) => (
        <Button
          variant="contained"
          size="small"
          onClick={() => (window.location.href = `${API_BASE}/download/${params.row.id}`)}
        >
          Download
        </Button>
      ),
    },
  ];

  useEffect(() => {
    fetchArtifacts();
  }, []);

  return (
    <Box>
      <Typography variant="h5" mb={2}>
        Upload Artifact
      </Typography>

      <Paper sx={{ p: 2, mb: 4, textAlign: "center", border: "1px dashed grey" }}>
        <Stack spacing={2} alignItems="center">
          <TextField type="file" onChange={(e: ChangeEvent<HTMLInputElement>) => setFile(e.target.files?.[0] || null)} />
          <Button variant="contained" onClick={upload} disabled={!file}>
            Upload
          </Button>
        </Stack>
      </Paper>

      <Typography variant="h5" mb={2}>
        Recent Artifacts
      </Typography>

      <Box height={500}>
        <DataGrid
          rows={artifacts}
          getRowId={(row: Artifact) => row.id}
          columns={columns}
          initialState={{
            pagination: {
              paginationModel: { pageSize: 5, page: 0 },
            },
          }}
          pageSizeOptions={[5, 10]}
        />
      </Box>
    </Box>
  );
}

export default Artifacts;