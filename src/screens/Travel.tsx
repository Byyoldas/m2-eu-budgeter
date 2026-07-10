/**
 * Step 6 — Travel (Category C1).
 * Supports Itemized trips (flight + accommodation + subsistence + domestic) and Flat Amount trips.
 * Live cost preview as user fills the form.
 */

import { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { useProjectStore, useTrips } from '../store/projectStore';
import { addTrip, updateTrip, deleteTrip, previewTripCost, getCountries } from '../ipc/commands';
import { useBudgetSummary, usePreview } from '../hooks/useBudgetSummary';
import { TripCard } from '../components/TripCard';
import { EmptyStateCard } from '../components/EmptyStateCard';
import { LivePreviewBox } from '../components/LivePreviewBox';
import type { TripDetailDto, TripCostPreviewDto, TripInput, CountrySummary } from '../types';

interface TravelProps {
  onNext: () => void;
  onBack: () => void;
}

type Mode = 'list' | 'add' | 'edit';
type TripKind = 'Itemized' | 'FlatAmount';

function fmt(v: string | null | undefined): string {
  if (!v) return '—';
  const n = parseFloat(v);
  return isNaN(n) ? '—' : `€ ${n.toLocaleString('en-GB', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

export function Travel({ onNext, onBack }: TravelProps) {
  const trips = useTrips();
  const projectConfig = useProjectStore((s) => s.projectConfig);
  const rateVersionId = projectConfig?.rate_version_id ?? 'from_2025_05_13';
  const duration = projectConfig?.duration_years ?? 5;
  const wpCount = projectConfig?.work_package_count ?? 1;
  const wpNames = projectConfig?.work_package_names ?? [];
  const storedCountries = useProjectStore((s) => s.countries);
  const setCountries = useProjectStore((s) => s.setCountries);

  const [mode, setMode] = useState<Mode>('list');
  const [tripKind, setTripKind] = useState<TripKind>('Itemized');
  const [editingTrip, setEditingTrip] = useState<TripDetailDto | null>(null);
  const [previewResult, setPreviewResult] = useState<TripCostPreviewDto | null>(null);

  const { mutate, isLoading } = useBudgetSummary();
  const { preview, isLoading: previewLoading } = usePreview<TripCostPreviewDto>();

  const { register, handleSubmit, watch, reset } = useForm({
    defaultValues: {
      name: '', project_year: 1, number_of_instances: 1,
      destination_country_code: '', one_way_distance_km: 0,
      number_of_nights: 1, number_of_days: 1,
      domestic_transport_per_instance_eur: '0',
      flat_amount_per_instance_eur: '',
      work_package_id: '',
    },
  });

  const watched = watch();

  useEffect(() => {
    if (storedCountries.length === 0) {
      getCountries(rateVersionId).then(setCountries).catch(() => {});
    }
  }, [rateVersionId]);

  useEffect(() => {
    const timer = setTimeout(async () => {
      const instances = Number(watched.number_of_instances);
      if (!instances) { setPreviewResult(null); return; }

      let input: TripInput | null = null;
      if (tripKind === 'Itemized') {
        if (!watched.destination_country_code) { setPreviewResult(null); return; }
        input = {
          name: watched.name, project_year: Number(watched.project_year),
          number_of_instances: instances, work_package_id: watched.work_package_id ? Number(watched.work_package_id) : null,
          trip_type: {
            Itemized: {
              destination_country_code: watched.destination_country_code,
              one_way_distance_km: Number(watched.one_way_distance_km),
              number_of_nights: Number(watched.number_of_nights),
              number_of_days: Number(watched.number_of_days),
              domestic_transport_per_instance_eur: watched.domestic_transport_per_instance_eur || '0',
            },
          },
        };
      } else {
        if (!parseFloat(watched.flat_amount_per_instance_eur)) { setPreviewResult(null); return; }
        input = {
          name: watched.name, project_year: Number(watched.project_year),
          number_of_instances: instances, work_package_id: watched.work_package_id ? Number(watched.work_package_id) : null,
          trip_type: { FlatAmount: { flat_amount_per_instance_eur: watched.flat_amount_per_instance_eur } },
        };
      }
      if (input) {
        const result = await preview(() => previewTripCost(input!));
        setPreviewResult(result);
      }
    }, 400);
    return () => clearTimeout(timer);
  }, [JSON.stringify(watched), tripKind]);

  const openAdd = () => { reset(); setEditingTrip(null); setPreviewResult(null); setTripKind('Itemized'); setMode('add'); };
  const openEdit = (trip: TripDetailDto) => {
    setEditingTrip(trip);
    const isItemized = trip.flight_cost_per_instance !== null;
    setTripKind(isItemized ? 'Itemized' : 'FlatAmount');
    reset({ name: trip.name, project_year: trip.project_year, number_of_instances: trip.number_of_instances });
    setPreviewResult(null);
    setMode('edit');
  };
  const handleDelete = async (id: string) => {
    if (!window.confirm('Delete this trip?')) return;
    await mutate(() => deleteTrip(id));
  };

  const onSubmit = async (data: typeof watched) => {
    let input: TripInput;
    if (tripKind === 'Itemized') {
      input = {
        name: data.name, project_year: Number(data.project_year),
        number_of_instances: Number(data.number_of_instances),
        work_package_id: data.work_package_id ? Number(data.work_package_id) : null,
        trip_type: {
          Itemized: {
            destination_country_code: data.destination_country_code,
            one_way_distance_km: Number(data.one_way_distance_km),
            number_of_nights: Number(data.number_of_nights),
            number_of_days: Number(data.number_of_days),
            domestic_transport_per_instance_eur: data.domestic_transport_per_instance_eur || '0',
          },
        },
      };
    } else {
      input = {
        name: data.name, project_year: Number(data.project_year),
        number_of_instances: Number(data.number_of_instances),
        work_package_id: data.work_package_id ? Number(data.work_package_id) : null,
        trip_type: { FlatAmount: { flat_amount_per_instance_eur: data.flat_amount_per_instance_eur } },
      };
    }
    const command = editingTrip ? () => updateTrip(editingTrip.id, input) : () => addTrip(input);
    const result = await mutate(command);
    if (result) setMode('list');
  };

  const previewRows: { label: string; value: string; highlight?: boolean }[] = previewResult
    ? tripKind === 'Itemized'
      ? [
          { label: 'Flight', value: fmt(previewResult.flight_cost_per_instance) },
          previewResult.flight_band_label ? { label: 'Band', value: previewResult.flight_band_label } : null,
          { label: 'Accommodation', value: fmt(previewResult.accommodation_cost_per_instance) },
          previewResult.accommodation_rate_eur ? { label: 'Acc. rate/night', value: fmt(previewResult.accommodation_rate_eur) } : null,
          { label: 'Subsistence', value: fmt(previewResult.subsistence_cost_per_instance) },
          { label: 'Domestic transport', value: fmt(previewResult.domestic_transport_per_instance) },
          { label: 'Per instance', value: fmt(previewResult.per_instance_total_eur) },
          { label: 'Total (all instances)', value: fmt(previewResult.total_trip_cost_eur), highlight: true },
        ].filter(Boolean) as { label: string; value: string; highlight?: boolean }[]
      : [
          { label: 'Per instance', value: fmt(previewResult.per_instance_total_eur) },
          { label: 'Total (all instances)', value: fmt(previewResult.total_trip_cost_eur), highlight: true },
        ]
    : [];

  if (mode !== 'list') {
    return (
      <div className="screen">
        <div className="screen-header">
          <h2 className="screen-title">{mode === 'add' ? 'Add Trip' : 'Edit Trip'}</h2>
          <div className="segmented-control">
            <button type="button" className={`seg-btn${tripKind === 'Itemized' ? ' seg-btn--active' : ''}`} onClick={() => setTripKind('Itemized')}>Itemized (EU rates)</button>
            <button type="button" className={`seg-btn${tripKind === 'FlatAmount' ? ' seg-btn--active' : ''}`} onClick={() => setTripKind('FlatAmount')}>Flat Amount</button>
          </div>
        </div>
        <div className="screen-split">
          <form className="screen-form" onSubmit={handleSubmit(onSubmit)} noValidate>
            <div className="form-section">
              <div className="form-field">
                <label htmlFor="trip-name" className="form-label required">Trip Name / Purpose</label>
                <input id="trip-name" type="text" placeholder="e.g. Conference EMNLP, Field work India" className="form-input" {...register('name')} />
              </div>
              <div className="form-row">
                <div className="form-field">
                  <label htmlFor="trip-year" className="form-label required">Project Year</label>
                  <select id="trip-year" className="form-input" {...register('project_year', { valueAsNumber: true })}>
                    {Array.from({ length: duration }, (_, i) => i + 1).map((y) => (
                      <option key={y} value={y}>Year {y}</option>
                    ))}
                  </select>
                </div>
                <div className="form-field">
                  <label htmlFor="trip-instances" className="form-label required">No. of Instances</label>
                  <input id="trip-instances" type="number" min={1} className="form-input" {...register('number_of_instances', { valueAsNumber: true })} />
                </div>
              </div>

              {tripKind === 'Itemized' && (
                <>
                  <div className="form-field">
                    <label htmlFor="country" className="form-label required">Destination Country</label>
                    <select id="country" className="form-input" {...register('destination_country_code')}>
                      <option value="">— Select country —</option>
                      {storedCountries.map((c: CountrySummary) => (
                        <option key={c.country_code} value={c.country_code}>{c.country_name}</option>
                      ))}
                    </select>
                  </div>
                  <div className="form-field">
                    <label htmlFor="distance" className="form-label required">One-Way Distance (km)</label>
                    <input id="distance" type="number" min={0} className="form-input" placeholder="e.g. 4800 for India" {...register('one_way_distance_km', { valueAsNumber: true })} />
                    <span className="form-hint">Drives flight band selection. Use 0 for local/train trips (no flight).</span>
                  </div>
                  <div className="form-row">
                    <div className="form-field">
                      <label htmlFor="nights" className="form-label required">Nights</label>
                      <input id="nights" type="number" min={1} className="form-input" {...register('number_of_nights', { valueAsNumber: true })} />
                    </div>
                    <div className="form-field">
                      <label htmlFor="days" className="form-label required">Days (subsistence)</label>
                      <input id="days" type="number" min={1} className="form-input" {...register('number_of_days', { valueAsNumber: true })} />
                    </div>
                  </div>
                  <div className="form-field">
                    <label htmlFor="domestic" className="form-label">Domestic Transport / instance (€)</label>
                    <input id="domestic" type="number" step="any" min={0} placeholder="0" className="form-input" {...register('domestic_transport_per_instance_eur')} />
                  </div>
                </>
              )}

              {tripKind === 'FlatAmount' && (
                <div className="form-field">
                  <label htmlFor="flat" className="form-label required">Flat Amount / instance (€)</label>
                  <input id="flat" type="number" step="any" min={0} className="form-input" {...register('flat_amount_per_instance_eur')} />
                </div>
              )}

              {wpCount > 0 && (
                <div className="form-field">
                  <label htmlFor="trip-wp" className="form-label">Work Package</label>
                  <select id="trip-wp" className="form-input" {...register('work_package_id')}>
                    <option value="">— None —</option>
                    {Array.from({ length: wpCount }, (_, i) => i + 1).map((wpId) => (
                      <option key={wpId} value={wpId}>{(wpNames[wpId - 1] as string | null) ?? `WP${wpId}`}</option>
                    ))}
                  </select>
                </div>
              )}
            </div>
            <div className="screen-footer">
              <button type="button" className="btn btn--ghost" onClick={() => setMode('list')}>Cancel</button>
              <button type="submit" className="btn btn--primary" disabled={isLoading}>
                {isLoading ? 'Saving…' : (editingTrip ? 'Update Trip' : 'Add Trip')}
              </button>
            </div>
          </form>
          <aside className="screen-aside">
            <LivePreviewBox title="Trip Cost Preview" rows={previewRows} isLoading={previewLoading} />
          </aside>
        </div>
      </div>
    );
  }

  return (
    <div className="screen">
      <div className="screen-header">
        <h2 className="screen-title">Travel (Category C1)</h2>
        <p className="screen-description">
          Costs calculated automatically from EU official rates (Annex 2a/2b).
          Select itemized for EU-rate trips or flat amount for pre-agreed costs.
        </p>
        <button className="btn btn--primary" onClick={openAdd}>+ Add Trip</button>
      </div>
      <div className="item-list">
        {trips.length === 0 ? (
          <EmptyStateCard icon="✈️" title="No trips yet"
            description="Add conferences, field work, collaboration visits, and other travel."
            action={{ label: '+ Add First Trip', onClick: openAdd }} />
        ) : (
          trips.map((trip) => (
            <TripCard key={trip.id} trip={trip} onEdit={openEdit} onDelete={handleDelete} />
          ))
        )}
      </div>
      <div className="screen-footer">
        <button className="btn btn--ghost" onClick={onBack}>← Back</button>
        <button className="btn btn--primary btn--lg" onClick={onNext}>Next: Other Costs →</button>
      </div>
    </div>
  );
}
