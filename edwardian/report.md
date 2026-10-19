# Edwardian platform-readiness report

- Target platform: **Windows**
- Overall readiness: **67%**
- Status: **BLOCKED**
- Blockers: 7
- Findings: 201, directives: 96, proposals: 0

## Readiness by dimension

| Dimension | Score | Findings | Highest severity |
|---|---|---|---|
| Architecture | 100% | 0 | - |
| Source | 80% | 2 | minor |
| Runtime | 100% | 0 | - |
| Build | 100% | 0 | - |
| Tooling | 0% | 108 | major |
| Filesystem | 0% | 81 | major |
| Process model | 0% | 9 | blocker |
| Native dependencies | 100% | 0 | - |
| CI | 90% | 1 | minor |
| Distribution | 100% | 0 | - |

## Blocker and major findings

- **FND-00163** [blocker / shell] invocation of an absolute unix shell path (2 evidence)
- **FND-00164** [blocker / service] systemd service integration (3 evidence)
- **FND-00169** [blocker / service] systemd service integration (2 evidence)
- **FND-00182** [blocker / shell] invocation of an absolute unix shell path (2 evidence)
- **FND-00187** [blocker / shell] invocation of an absolute unix shell path (1 evidence)
- **FND-00196** [blocker / service] systemd service integration (1 evidence)
- **FND-00197** [blocker / service] systemd service integration (1 evidence)
- **FND-00001** [major / filesystem] hard-coded POSIX absolute path (/home/...) (1 evidence)
- **FND-00002** [major / filesystem] hard-coded POSIX absolute path (/home/...) (1 evidence)
- **FND-00015** [major / filesystem] hard-coded POSIX absolute path (/home/...) (8 evidence)
- **FND-00017** [major / process] POSIX signal handling (SIGTERM) (1 evidence)
- **FND-00018** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00020** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00022** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00024** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00026** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00028** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00030** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00032** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00034** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00036** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00038** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00040** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00042** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00044** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00046** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00048** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00050** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00052** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00054** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00056** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00058** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00060** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00062** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00064** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00066** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00068** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00070** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00072** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00074** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00076** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00078** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00080** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00082** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00084** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00086** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00088** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00090** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00092** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00094** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00096** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00098** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00100** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00102** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00104** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00106** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00108** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00110** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00112** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00114** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00116** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00118** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00120** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00122** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00124** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00126** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00128** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00130** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00132** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00134** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00136** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00138** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00140** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00142** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00144** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00146** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00148** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00150** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00152** [major / filesystem] hard-coded POSIX absolute path (/home/...) (3 evidence)
- **FND-00155** [major / tooling] pkg-config native library discovery (3 evidence)
- **FND-00156** [major / filesystem] hard-coded POSIX absolute path (/home/...) (8 evidence)
- **FND-00158** [major / filesystem] symlink creation or reliance (`symlink(...)`) (1 evidence)
- **FND-00159** [major / filesystem] unix permission model (`PermissionsExt`) (1 evidence)
- **FND-00160** [major / filesystem] unix mode literal (`0o755`) (1 evidence)
- **FND-00165** [major / service] SysV init script dependency (1 evidence)
- **FND-00166** [major / tooling] pkg-config native library discovery (1 evidence)
- **FND-00172** [major / tooling] pkg-config native library discovery (3 evidence)
- **FND-00175** [major / tooling] pkg-config native library discovery (1 evidence)
- **FND-00183** [major / filesystem] symlink creation or reliance (`symlink(...)`) (1 evidence)
- **FND-00184** [major / filesystem] unix permission model (`PermissionsExt`) (2 evidence)
- **FND-00185** [major / filesystem] unix mode literal (`0o755`) (1 evidence)

## Dependency chains

1. **EWD-D001** (P0) Remove invocation of an absolute unix shell path (blocker readiness impact) — no predecessors
2. **EWD-D002** (P0) Remove systemd service integration (blocker readiness impact) — after: EWD-D001
3. **EWD-D003** (P0) Remove systemd service integration (blocker readiness impact) — no predecessors
4. **EWD-D004** (P0) Remove invocation of an absolute unix shell path (blocker readiness impact) — no predecessors
5. **EWD-D005** (P0) Remove invocation of an absolute unix shell path (blocker readiness impact) — no predecessors
6. **EWD-D006** (P0) Remove systemd service integration (blocker readiness impact) — no predecessors
7. **EWD-D007** (P0) Remove systemd service integration (blocker readiness impact) — no predecessors
8. **EWD-D008** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — no predecessors
9. **EWD-D009** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D008
10. **EWD-D010** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — no predecessors
11. **EWD-D011** (P1) Remove POSIX signal handling (SIGTERM) (major readiness impact) — after: EWD-D010
12. **EWD-D012** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — no predecessors
13. **EWD-D013** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012
14. **EWD-D014** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013
15. **EWD-D015** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014
16. **EWD-D016** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015
17. **EWD-D017** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016
18. **EWD-D018** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017
19. **EWD-D019** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018
20. **EWD-D020** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019
21. **EWD-D021** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020
22. **EWD-D022** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021
23. **EWD-D023** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022
24. **EWD-D024** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023
25. **EWD-D025** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024
26. **EWD-D026** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025
27. **EWD-D027** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026
28. **EWD-D028** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027
29. **EWD-D029** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028
30. **EWD-D030** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029
31. **EWD-D031** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030
32. **EWD-D032** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031
33. **EWD-D033** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032
34. **EWD-D034** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033
35. **EWD-D035** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034
36. **EWD-D036** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035
37. **EWD-D037** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036
38. **EWD-D038** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037
39. **EWD-D039** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038
40. **EWD-D040** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039
41. **EWD-D041** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040
42. **EWD-D042** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041
43. **EWD-D043** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042
44. **EWD-D044** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043
45. **EWD-D045** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044
46. **EWD-D046** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045
47. **EWD-D047** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046
48. **EWD-D048** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047
49. **EWD-D049** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048
50. **EWD-D050** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049
51. **EWD-D051** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050
52. **EWD-D052** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051
53. **EWD-D053** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052
54. **EWD-D054** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053
55. **EWD-D055** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054
56. **EWD-D056** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055
57. **EWD-D057** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056
58. **EWD-D058** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057
59. **EWD-D059** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058
60. **EWD-D060** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059
61. **EWD-D061** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060
62. **EWD-D062** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061
63. **EWD-D063** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062
64. **EWD-D064** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063
65. **EWD-D065** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064
66. **EWD-D066** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064, EWD-D065
67. **EWD-D067** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064, EWD-D065, EWD-D066
68. **EWD-D068** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064, EWD-D065, EWD-D066, EWD-D067
69. **EWD-D069** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064, EWD-D065, EWD-D066, EWD-D067, EWD-D068
70. **EWD-D070** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064, EWD-D065, EWD-D066, EWD-D067, EWD-D068, EWD-D069
71. **EWD-D071** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064, EWD-D065, EWD-D066, EWD-D067, EWD-D068, EWD-D069, EWD-D070
72. **EWD-D072** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064, EWD-D065, EWD-D066, EWD-D067, EWD-D068, EWD-D069, EWD-D070, EWD-D071
73. **EWD-D073** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064, EWD-D065, EWD-D066, EWD-D067, EWD-D068, EWD-D069, EWD-D070, EWD-D071, EWD-D072
74. **EWD-D074** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064, EWD-D065, EWD-D066, EWD-D067, EWD-D068, EWD-D069, EWD-D070, EWD-D071, EWD-D072, EWD-D073
75. **EWD-D075** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064, EWD-D065, EWD-D066, EWD-D067, EWD-D068, EWD-D069, EWD-D070, EWD-D071, EWD-D072, EWD-D073, EWD-D074
76. **EWD-D076** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064, EWD-D065, EWD-D066, EWD-D067, EWD-D068, EWD-D069, EWD-D070, EWD-D071, EWD-D072, EWD-D073, EWD-D074, EWD-D075
77. **EWD-D077** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064, EWD-D065, EWD-D066, EWD-D067, EWD-D068, EWD-D069, EWD-D070, EWD-D071, EWD-D072, EWD-D073, EWD-D074, EWD-D075, EWD-D076
78. **EWD-D078** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064, EWD-D065, EWD-D066, EWD-D067, EWD-D068, EWD-D069, EWD-D070, EWD-D071, EWD-D072, EWD-D073, EWD-D074, EWD-D075, EWD-D076, EWD-D077
79. **EWD-D079** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D012, EWD-D013, EWD-D014, EWD-D015, EWD-D016, EWD-D017, EWD-D018, EWD-D019, EWD-D020, EWD-D021, EWD-D022, EWD-D023, EWD-D024, EWD-D025, EWD-D026, EWD-D027, EWD-D028, EWD-D029, EWD-D030, EWD-D031, EWD-D032, EWD-D033, EWD-D034, EWD-D035, EWD-D036, EWD-D037, EWD-D038, EWD-D039, EWD-D040, EWD-D041, EWD-D042, EWD-D043, EWD-D044, EWD-D045, EWD-D046, EWD-D047, EWD-D048, EWD-D049, EWD-D050, EWD-D051, EWD-D052, EWD-D053, EWD-D054, EWD-D055, EWD-D056, EWD-D057, EWD-D058, EWD-D059, EWD-D060, EWD-D061, EWD-D062, EWD-D063, EWD-D064, EWD-D065, EWD-D066, EWD-D067, EWD-D068, EWD-D069, EWD-D070, EWD-D071, EWD-D072, EWD-D073, EWD-D074, EWD-D075, EWD-D076, EWD-D077, EWD-D078
80. **EWD-D080** (P1) Remove pkg-config native library discovery (major readiness impact) — no predecessors
81. **EWD-D081** (P1) Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — after: EWD-D001, EWD-D002
82. **EWD-D082** (P1) Remove symlink creation or reliance (`symlink(...)`) (major readiness impact) — after: EWD-D001, EWD-D002, EWD-D081
83. **EWD-D083** (P1) Remove unix permission model (`PermissionsExt`) (major readiness impact) — after: EWD-D001, EWD-D002, EWD-D081, EWD-D082
84. **EWD-D084** (P1) Remove unix mode literal (`0o755`) (major readiness impact) — after: EWD-D001, EWD-D002, EWD-D081, EWD-D082, EWD-D083
85. **EWD-D085** (P1) Remove SysV init script dependency (major readiness impact) — after: EWD-D001, EWD-D002, EWD-D081, EWD-D082, EWD-D083, EWD-D084
86. **EWD-D086** (P1) Remove pkg-config native library discovery (major readiness impact) — after: EWD-D001, EWD-D002, EWD-D081, EWD-D082, EWD-D083, EWD-D084, EWD-D085
87. **EWD-D087** (P1) Remove pkg-config native library discovery (major readiness impact) — after: EWD-D003
88. **EWD-D088** (P1) Remove pkg-config native library discovery (major readiness impact) — after: EWD-D003, EWD-D087
89. **EWD-D089** (P1) Remove symlink creation or reliance (`symlink(...)`) (major readiness impact) — after: EWD-D005
90. **EWD-D090** (P1) Remove unix permission model (`PermissionsExt`) (major readiness impact) — after: EWD-D005, EWD-D089
91. **EWD-D091** (P1) Remove unix mode literal (`0o755`) (major readiness impact) — after: EWD-D005, EWD-D089, EWD-D090
92. **EWD-D092** (P2) Remove GitHub Actions CI (Linux-only unless a windows matrix exists) (minor readiness impact) — no predecessors
93. **EWD-D093** (P2) Remove unix-style path join in string literal (minor readiness impact) — after: EWD-D001, EWD-D002, EWD-D081, EWD-D082, EWD-D083, EWD-D084, EWD-D085, EWD-D086
94. **EWD-D094** (P2) Remove unix environment variable / user identity assumption (minor readiness impact) — after: EWD-D001, EWD-D002, EWD-D081, EWD-D082, EWD-D083, EWD-D084, EWD-D085, EWD-D086, EWD-D093
95. **EWD-D095** (P2) Remove unix-style path join in string literal (minor readiness impact) — no predecessors
96. **EWD-D096** (P2) Remove unix environment variable / user identity assumption (minor readiness impact) — after: EWD-D003, EWD-D087, EWD-D088

## Directives

- **EWD-D001** (P0): Remove invocation of an absolute unix shell path (blocker readiness impact) — 5 validation step(s)
- **EWD-D002** (P0): Remove systemd service integration (blocker readiness impact) — 5 validation step(s)
- **EWD-D003** (P0): Remove systemd service integration (blocker readiness impact) — 5 validation step(s)
- **EWD-D004** (P0): Remove invocation of an absolute unix shell path (blocker readiness impact) — 5 validation step(s)
- **EWD-D005** (P0): Remove invocation of an absolute unix shell path (blocker readiness impact) — 5 validation step(s)
- **EWD-D006** (P0): Remove systemd service integration (blocker readiness impact) — 5 validation step(s)
- **EWD-D007** (P0): Remove systemd service integration (blocker readiness impact) — 5 validation step(s)
- **EWD-D008** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D009** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D010** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D011** (P1): Remove POSIX signal handling (SIGTERM) (major readiness impact) — 5 validation step(s)
- **EWD-D012** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D013** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D014** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D015** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D016** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D017** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D018** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D019** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D020** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D021** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D022** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D023** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D024** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D025** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D026** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D027** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D028** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D029** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D030** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D031** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D032** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D033** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D034** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D035** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D036** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D037** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D038** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D039** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D040** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D041** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D042** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D043** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D044** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D045** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D046** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D047** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D048** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D049** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D050** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D051** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D052** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D053** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D054** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D055** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D056** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D057** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D058** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D059** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D060** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D061** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D062** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D063** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D064** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D065** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D066** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D067** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D068** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D069** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D070** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D071** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D072** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D073** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D074** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D075** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D076** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D077** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D078** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D079** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D080** (P1): Remove pkg-config native library discovery (major readiness impact) — 5 validation step(s)
- **EWD-D081** (P1): Remove hard-coded POSIX absolute path (/home/...) (major readiness impact) — 5 validation step(s)
- **EWD-D082** (P1): Remove symlink creation or reliance (`symlink(...)`) (major readiness impact) — 5 validation step(s)
- **EWD-D083** (P1): Remove unix permission model (`PermissionsExt`) (major readiness impact) — 5 validation step(s)
- **EWD-D084** (P1): Remove unix mode literal (`0o755`) (major readiness impact) — 5 validation step(s)
- **EWD-D085** (P1): Remove SysV init script dependency (major readiness impact) — 5 validation step(s)
- **EWD-D086** (P1): Remove pkg-config native library discovery (major readiness impact) — 5 validation step(s)
- **EWD-D087** (P1): Remove pkg-config native library discovery (major readiness impact) — 5 validation step(s)
- **EWD-D088** (P1): Remove pkg-config native library discovery (major readiness impact) — 5 validation step(s)
- **EWD-D089** (P1): Remove symlink creation or reliance (`symlink(...)`) (major readiness impact) — 5 validation step(s)
- **EWD-D090** (P1): Remove unix permission model (`PermissionsExt`) (major readiness impact) — 5 validation step(s)
- **EWD-D091** (P1): Remove unix mode literal (`0o755`) (major readiness impact) — 5 validation step(s)
- **EWD-D092** (P2): Remove GitHub Actions CI (Linux-only unless a windows matrix exists) (minor readiness impact) — 3 validation step(s)
- **EWD-D093** (P2): Remove unix-style path join in string literal (minor readiness impact) — 3 validation step(s)
- **EWD-D094** (P2): Remove unix environment variable / user identity assumption (minor readiness impact) — 3 validation step(s)
- **EWD-D095** (P2): Remove unix-style path join in string literal (minor readiness impact) — 3 validation step(s)
- **EWD-D096** (P2): Remove unix environment variable / user identity assumption (minor readiness impact) — 3 validation step(s)

## Proposals

_None._

