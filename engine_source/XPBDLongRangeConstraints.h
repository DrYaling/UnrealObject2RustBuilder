// Copyright Epic Games, Inc. All Rights Reserved.
#pragma once

#include "Chaos/PBDLongRangeConstraintsBase.h"
#include "Chaos/PBDParticles.h"
#include "ChaosStats.h"

DECLARE_CYCLE_STAT(TEXT("Chaos XPBD Long Range Constraint"), STAT_XPBD_LongRange, STATGROUP_Chaos);

namespace Chaos
{
// Stiffness is in N/CM^2, so it needs to be adjusted from the PBD stiffness ranging between [0,1]
static const double XPBDLongRangeMaxCompliance = 1.e-3;

template<class T, int d>
class TXPBDLongRangeConstraints final : public TPBDLongRangeConstraintsBase<T, d>
{
public:
	typedef TPBDLongRangeConstraintsBase<T, d> Base;
	typedef typename Base::FTether FTether;
	typedef typename Base::EMode EMode;

	TXPBDLongRangeConstraints(
		const TPBDParticles<T, d>& Particles,
		const TMap<int32, TSet<int32>>& PointToNeighbors,
		const int32 NumberOfAttachments = 4,
		const T InStiffness = (T)1,
		const T LimitScale = (T)1,
		const EMode InMode = EMode::Geodesic)
	    : TPBDLongRangeConstraintsBase<T, d>(Particles, PointToNeighbors, NumberOfAttachments, InStiffness, LimitScale, InMode)
	{
		Lambdas.Reserve(Tethers.Num());
	}

	~TXPBDLongRangeConstraints() {}

	void Init() const
	{
		Lambdas.Reset();
		Lambdas.AddZeroed(Tethers.Num());
	}

	void Apply(TPBDParticles<T, d>& Particles, const T Dt) const 
	{
		SCOPE_CYCLE_COUNTER(STAT_XPBD_LongRange);
		// Run particles in parallel, and ranges in sequence to avoid a race condition when updating the same particle from different tethers
		static const int32 MinParallelSize = 500;
		TethersView.ParallelFor([this, &Particles, Dt](TArray<FTether>& /*InTethers*/, int32 Index)
			{
				Apply(Particles, Dt, Index);
			}, MinParallelSize);
	}

	void Apply(TPBDParticles<T, d>& Particles, const T Dt, const TArray<int32>& ConstraintIndices) const
	{
		SCOPE_CYCLE_COUNTER(STAT_XPBD_LongRange);
		for (const int32 Index : ConstraintIndices)
		{
			Apply(Particles, Dt, Index);
		}
	}

private:
	void Apply(TPBDParticles<T, d>& Particles, const T Dt, int32 Index) const
	{
		const FTether& Tether = Tethers[Index];

		TVector<T, d> Direction;
		T Offset;
		Tether.GetDelta(Particles, Direction, Offset);

		T& Lambda = Lambdas[Index];
		const T Alpha = (T)XPBDLongRangeMaxCompliance / (Stiffness * Dt * Dt);

		const T DLambda = (Offset - Alpha * Lambda) / ((T)1. + Alpha);
		Particles.P(Tether.End) += DLambda * Direction;
		Lambda += DLambda;
	}

private:
	using Base::Tethers;
	using Base::TethersView;
	using Base::Stiffness;

	mutable TArray<T> Lambdas;
};
}
